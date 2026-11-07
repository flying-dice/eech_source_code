-- TEST-ONLY host-owned witness of the two REQUIRED sector-height behaviours the finite native gates did not reach (EEDCS-49, D3, Task49): ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1 (M5, the guarded fixed-wing
-- clearance) and ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1 (M6, the native server-session save and the campaign's own load). One bounded process per mode, fresh installation root, never repeated into a green.
-- Contract and plan: docs/d3-sector-height-production-contract.md sections 8 and 12. Root runs it natively after the SAME source review; nothing here is a native result, and the scripted tests of this file (tests/
-- sector_height_witness_driver.rs) are boundary checks of THIS driver, never campaign authority.
--
-- Boundary under test (the product path): this host-owned Lua -> the public native module (`require ("eech_dc")`, `dc.boot`, `engine:frame`, and for M6 the session method `sector_height_witness_save`) -> the REAL campaign
-- code, which asks THIS host for the sector's word (scan phase 17) and states a saved word to it (phase 18) through the one bridge the ordinary build ships. The host's own answers are the real provider's (`host.model`), delegated to
-- unchanged and counted; a mode alters ONE eligible intervention and nothing else. Nothing is searched: no actor, ray, seed or window is chosen by what a run reached (the window of an M5 run is named by the caller BEFORE the
-- launch; an absent case is UNREACHED, never a pass).
--
-- M5, the guarded clearance (needs a DLL built with the ordinary hosted profile; `eech-dc/sector-height-witness` is NOT needed). In a hosted build the campaign reads the host's word ONLY inside the guarded clearance statement of
-- fw_move.c (fixed_wing_movement_waypoint: a task leader whose guide follows terrain) and outside the scan and the session pack (the census guard of tests/sector_height_guard.rs pins every reader). So a phase 17 question that arrives
-- while NO scan enumeration is open (phases 0 to 3 up to the scan's `done_*` reply) and during no session save IS the guarded clearance asking: that is the ELIGIBILITY, derived from the campaign's own call structure and not guessed.
-- A WINDOW is a maximal run of such questions with no other host callback between: a task leader's step reads the word twice (the wrapper that discards the answer, then the navigation that hands the minimum height to the host),
-- with nothing between them. The hand-off is the `fixed_wing_move` step carrying `fixed_wing_terrain` that is the very next host callback after a `fixed_wing_lookahead` of the SAME host id, which itself is the very next host callback
-- after the window: any other callback (another vehicle, a model question, a bind, a loss, a scan read) between them drops the relation, so a hand-off is credited only for a complete same-caller sequence read, look-ahead, move with
-- the host's REAL positive answer to both. Its `min_height` is the campaign's own result of the ORIGINAL expressions max(terrain elevation, word - terrain elevation, waypoint altitude) under the reference host's accepted SSE
-- round-toward-zero profile (the host's reference arithmetic selection), and `elevation` is that terrain elevation.
-- The caller of a READ cannot be known when the read arrives (the read carries no identity): an intervention is therefore made only on a window whose recorded TRAJECTORY (window ordinal, frame, windows without a hand-off, scan reads,
-- hand-offs so far) equals the successful paired control's, and after the consumption the hand-off's host id, elevation, pre height and phase must equal the control's. No leader is created and no actor is hunted.
--
--   clearance_original         the same declared window in the ORIGINAL-solver profile (a build that keeps the original entity solver: `eech-dc/collision-fixture`, no witness feature): the campaign consumes its own legacy word and the host is
--                              never asked for a sector height (a run in which it was is FAILURE: not that profile). Runs until the FIRST hand-off and reports it (the word itself is not observable in this profile): the characterisation FIRST.
--   clearance_control          nothing is altered; `original=<the successful clearance_original report>` makes it PAIRED: its first hand-off must equal the original profile's, field for field, and the two reports must be whole, successful and of the
--                              same setup. Without `original=` it is explicitly UNPAIRED (a standalone control: no parity claim, and no intervention mode accepts it). Runs until the FIRST window followed by a complete same-caller hand-off.
--   clearance_changed_word     `window=<N>` and `control=<the successful PAIRED control report>` (the same setup; N must be its window): EVERY word read in window N is answered 100000.0 (0x47c35000, finite and dominant). PASS needs the
--                              hand-off of window N to equal the accepted round-toward-zero single precision value of 100000.0 - elevation exactly, from the control's own caller; `physical=changed|unchanged` is stated against the control's answered height.
--   clearance_withheld         the host raises at the FIRST read of window N:                                  [model-provider:entity_scan.callback]
--   clearance_malformed        two patterns instead of three:                                                  [model-provider:entity_scan.answer]
--   clearance_wrong_state      `restored` for a read:                                                          [model-provider:entity_scan.state]
--   clearance_wrong_sector     the sector echoed one unit away:                                                [model-provider:entity_scan.sector_height]
--   A refusal mode is credited ONLY with the one eligible intervention actually made AND the engine stopping at exactly that marker after it; any fatal before it, a run that ends without a fatal, or another marker is a FAILURE; a
--   window never reached is UNREACHED, and a window whose trajectory differs from the control's is never intervened on. A fatal, a request fault or a reply fault always takes precedence over a clean UNREACHED.
--
-- M6, the native save and the campaign's own load (needs a DLL built with `eech-dc/sector-height-witness`; ordinary shipping exports unchanged). The save is the campaign's own `store_session` (uisession.c) over the live session,
-- called by the fixture method: it writes the saved script and the saved data (the session version, then pack_session PACK_MODE_SERVER_SESSION), and packing a sector asks the host for its word. `store_session` RETRIES
-- `pack_session` with a larger buffer while it reports an overflow, and each attempt reads every sector again: the driver reconciles these ATTEMPTS (a repeat of a sector read starts a new attempt; every attempt must repeat the
-- first one's sectors in order with the same words, else the save is unsupported repetition) and keeps the unique final saved-sector relationship. The load is the campaign's own too: a LATER BOOT of the saved script
-- (parsgen.c FILE_TAG_SAVED_CAMPAIGN -> unpack_session), whose sector unpack states each saved word RAW to the host (phase 18), the host holding exactly those bits. No method of this driver packs, unpacks or restores for the
-- campaign. Seeds are the host's explicit raw HISTORIES (an INPUT): before the save the driver states them to its OWN provider as a restore (the host's world holding a history), never to the campaign.
--
--   save                       args: name=<1 to 24 of A-Z a-z 0-9 _ ->, seeds=<sx:sz:0xWORD,...> (optional; raw words: negative, infinite and quiet NaN are valid history, a SIGNALLING NaN is refused: unmeasured), frames (default 0).
--                              Boots, seeds, makes the campaign save, and records every word the pack asked for (`sh-save-word`, the unique final relationship; `sh-witness-pack` the attempts). PASS needs the save line, every seeded
--                              sector packed with its seed's bits, at least one sector and consistent attempts.
--   save_withheld              the host raises at the FIRST pack read of the save: [model-provider:entity_scan.callback] (a fault after the eligible read; a fatal before it is a FAILURE)
--   load                       args: campaign=<the saved script's file name, `<save name>.S..`>, expect=<the SUCCESSFUL save run's report: same setup, whole, unique ordered records, terminal PASS>, name=<a NEW saved name for the second
--                              pack>, seeds=<as in the save run> (optional). Boots the saved script (the campaign's load) and records every restore. PASS needs: every restore stated exactly the word the save packed for that sector (and no
--                              extra, none missing), the host acknowledged those bits, every restore BEFORE the first entity insertion (restore-before-insert), then the campaign saves again (`name`, consistent attempts) and no sector's
--                              final word is LOWER than the word restored for it (a NaN unchanged, an infinity unchanged unless raised, a negative history raised only upward).
--   load_withheld / load_wrong_sector / load_wrong_state / load_stale_word
--                              ONE fault at the FIRST restore, the campaign's own boot stopping at the named marker: callback / entity_scan.sector_height (a restore echoed for another sector) / entity_scan.state (`height` for a
--                              restore) / entity_scan.sector_height (the host stored a word that is not the one stated). Positive-first: each is the single fault of the same boot as `load` (same `expect=` validation).
--
-- Every report this driver consumes (original=, control=, expect=) is read as a WHOLE successful report of this driver version (schema 2): known records only, unique single records, ordered rows, no fault record, a terminal
-- PASS of the expected mode and profile, and the same scenario, seed, frame_ms and hours; anything else refuses before any side effect. Every host request and reply it credits is checked as the exact raw grammar of the public
-- C bridge: comma lists of exactly the stated 0x%08x patterns (an empty or extra comma segment is malformed), no placement name, the sector echoed, a restore's word acknowledged.
--
-- host.args: root, out (the report; created EXCLUSIVELY first), mode, scenario (lebanon_retail default, georgia_retail), map, campaign (a load's saved script), hours (M5 default 3: the accepted regression-pack window, 3
-- simulated hours, 100 ms frames, seed 1; named here, before any launch), frame_ms (100), seed (1), window, original, control, name, seeds, expect, frames.
-- NOTHING IN THIS FILE DIVIDES BY A VALUE THAT CAN BE ZERO (a divide by zero is a hardware trap in the native host that pcall cannot catch): the one division is the frame count by `frame_ms`, asserted positive. The float
-- helpers below are the same block as entity-authority.lua's (tests/sector_height_witness_driver.rs compares them); the single-precision arithmetic and decoding use frexp/ldexp and multiplication only.

local NAME = "sector-height-witness"
local function refuse (message) error (NAME .. ": " .. message, 0) end

local huge = math.huge
local args = host.args
local root = assert (args.root, "root=<installation root>")
local out_path = assert (args.out, "out=<report file>")
local mode = assert (args.mode, "mode=<a mode of the header>")

-- BEGIN words
-- ---------------------------------------------------------------------------------------------------------------- float patterns
-- NOTHING HERE DIVIDES. The native host's floating point policy after boot traps a divide by zero in hardware (the sign test `1 / x < 0` of a zero ended the rotor witness's public controls with
-- 0xC000008E, and `pcall` cannot catch a trap), so the sign of a zero is read from its printed form and the two halves of a word are taken by multiplying with an exact power of two.
-- This block is the SAME text as the one in tree-authority.lua (tests/entity_authority.rs compares the two), which runs it with the trap enabled in a child process.
local INVERSE_65536 = 0.0000152587890625 -- 2^-16, exact
local SUBNORMAL_SCALE = 2 ^ 149 -- exact
local function hex8 (bits)
	local high = math.floor (bits * INVERSE_65536)
	return string.format ("%04x%04x", high, bits - high * 65536)
end
-- the IEEE single word of an exact single (eight hex digits), or nil for a number that is not one
local function word_of (x)
	if type (x) ~= "number" or x ~= x then return nil end
	local negative = x < 0 or (x == 0 and string.sub (string.format ("%g", x), 1, 1) == "-")
	if negative then x = -x end
	local bits
	if x == huge then
		bits = 0x7f800000
	elseif x == 0 then
		bits = 0
	else
		local m, e = math.frexp (x)
		local exponent = e + 126
		local mantissa
		if exponent >= 1 then
			if exponent >= 255 then return nil end
			mantissa = (m * 2 - 1) * 8388608
		else
			exponent = 0
			mantissa = x * SUBNORMAL_SCALE
		end
		if mantissa ~= math.floor (mantissa) or mantissa < 0 or mantissa >= 8388608 then return nil end
		bits = exponent * 8388608 + mantissa
	end
	if negative then bits = bits + 2147483648 end
	return hex8 (bits)
end
-- END words

local ZERO = "0x00000000"
local PAT = "0x%x%x%x%x%x%x%x%x"

local MODES = {
	clearance_original = { family = "clearance", original = true, profile = "original-solver" },
	clearance_control = { family = "clearance", profile = "hosted" },
	clearance_changed_word = { family = "clearance", intervene = "word", profile = "hosted" },
	clearance_withheld = { family = "clearance", intervene = "fault", marker = "entity_scan.callback", profile = "hosted" },
	clearance_malformed = { family = "clearance", intervene = "fault", marker = "entity_scan.answer", profile = "hosted" },
	clearance_wrong_state = { family = "clearance", intervene = "fault", marker = "entity_scan.state", profile = "hosted" },
	clearance_wrong_sector = { family = "clearance", intervene = "fault", marker = "entity_scan.sector_height", profile = "hosted" },
	save = { family = "session", profile = "witness-feature" },
	save_withheld = { family = "session", intervene = "fault", marker = "entity_scan.callback", at = "pack", profile = "witness-feature" },
	load = { family = "session", profile = "witness-feature" },
	load_withheld = { family = "session", intervene = "fault", marker = "entity_scan.callback", at = "restore", profile = "witness-feature" },
	load_wrong_sector = { family = "session", intervene = "fault", marker = "entity_scan.sector_height", at = "restore", profile = "witness-feature" },
	load_wrong_state = { family = "session", intervene = "fault", marker = "entity_scan.state", at = "restore", profile = "witness-feature" },
	load_stale_word = { family = "session", intervene = "fault", marker = "entity_scan.sector_height", at = "restore", profile = "witness-feature" },
}
local plan = MODES [mode]
if plan == nil then refuse ("mode=" .. tostring (mode) .. " is not a mode of this witness") end
local loading = mode == "load" or string.sub (mode, 1, 5) == "load_"
local saving = mode == "save" or mode == "save_withheld"

local hours = tonumber (args.hours or (plan.family == "clearance" and "3" or "0"))
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
local target_window
if plan.family == "clearance" and plan.intervene then
	target_window = tonumber (args.window)
	if target_window == nil or target_window < 1 or target_window ~= math.floor (target_window) then refuse ("mode " .. mode .. " needs window=<N>, the ordinal of the control's first hand-off window") end
elseif args.window ~= nil then
	refuse ("window= is only for a clearance mode that intervenes")
end
if args.original ~= nil and mode ~= "clearance_control" then refuse ("original=<report> is only for clearance_control") end
if args.control ~= nil and not (plan.family == "clearance" and plan.intervene) then refuse ("control=<report> is only for a clearance mode that intervenes") end
if plan.family == "clearance" and plan.intervene and (type (args.control) ~= "string" or args.control == "") then refuse ("mode " .. mode .. " needs control=<the successful paired clearance_control report>") end
if args.original ~= nil and (type (args.original) ~= "string" or args.original == "") then refuse ("original=<the clearance_original report>") end

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

-- the host's raw histories (M6): sx:sz:0xWORD, comma separated; a SIGNALLING NaN is refused (unmeasured), every other word is valid history
local function bits_of (text) return tonumber (string.sub (text, 3), 16) end
local function signalling_nan (text)
	local n = bits_of (text)
	return (n >= 0x7f800001 and n < 0x7fc00000) or (n >= 0xff800001 and n < 0xffc00000)
end
local seeds, seed_order = {}, {}
if args.seeds ~= nil and args.seeds ~= "" then
	if plan.family ~= "session" then refuse ("seeds= is only for a session mode") end
	local position, total = 1, #args.seeds
	while true do
		local item = string.match (args.seeds, "^[^,]*", position)
		position = position + #item
		local x, z, word = string.match (item, "^(%d+):(%d+):(" .. PAT .. ")$")
		if x == nil then refuse ("a seed is sx:sz:0xWORD with a whole sector index and eight hex digits, got \"" .. item .. "\"") end
		if signalling_nan (word) then refuse ("the seed " .. item .. " is a signalling NaN: unmeasured, never seeded") end
		local key = x .. "," .. z
		if seeds [key] ~= nil then refuse ("the sector " .. key .. " is seeded twice") end
		seeds [key] = { x = tonumber (x), z = tonumber (z), word = string.lower (word) }
		seed_order [#seed_order + 1] = key
		if position > total then break end
		position = position + 1 -- the comma; an item must follow it
		if position > total then refuse ("the seeds end with a comma: an empty segment is malformed") end
	end
end
local save_name = args.name
if plan.family == "session" and (mode == "save" or mode == "save_withheld" or mode == "load") then
	if type (save_name) ~= "string" or not string.match (save_name, "^[%w_%-]+$") or #save_name > 24 then refuse ("name=<1 to 24 of A-Z a-z 0-9 _ -> is required") end
end
if loading then
	if type (args.campaign) ~= "string" or args.campaign == "" then refuse ("a load needs campaign=<the saved script's file name>") end
	if type (args.expect) ~= "string" or args.expect == "" then refuse ("mode " .. mode .. " needs expect=<the successful save run's report>") end
elseif args.expect ~= nil then
	refuse ("expect=<report> is only for a load mode")
end
local frames_arg = tonumber (args.frames or "0")
assert (frames_arg and frames_arg >= 0 and frames_arg == math.floor (frames_arg), "frames must be a whole number")

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
local KNOWN = {
	["sh-witness-identity"] = "one", ["sh-witness-counts"] = "one", ["sh-witness-handoff"] = "one", ["sh-witness-trajectory"] = "one", ["sh-witness-original"] = "one", ["sh-witness-effect"] = "one",
	["sh-witness-save"] = "one", ["sh-witness-pack"] = "one", ["sh-save-seed"] = "many", ["sh-save-word"] = "many", ["sh-load-restore"] = "many", ["sh-witness-verdict"] = "one",
}
local REFUSED_RECORD = { ["sh-witness-fatal"] = true, ["sh-witness-reply-fault"] = true, ["sh-witness-request-fault"] = true }
-- the whole successful report of this driver at `path` for `role` (original | control | save), or a refusal naming what is missing or inconsistent with THIS run's setup
local function read_reference (path, role, expected_mode)
	local file = io.open (path, "rb")
	if file == nil then refuse ("cannot read the " .. role .. " report " .. path) end
	local rec, order, last = {}, {}, nil
	for line in file:lines () do
		line = string.gsub (line, "\r$", "")
		local name = string.match (line, "^(%S+)")
		if name == nil then file:close () refuse ("the " .. role .. " report holds an empty line") end
		if REFUSED_RECORD [name] then file:close () refuse ("the " .. role .. " report holds a " .. name .. " record: it is not a successful report") end
		local kind = KNOWN [name]
		if kind == nil then file:close () refuse ("the " .. role .. " report holds the unknown record " .. name) end
		local fields
		if name == "sh-witness-verdict" then
			local m, s, detail = string.match (line, "^sh%-witness%-verdict mode=(%S+) status=(%S+) detail=(.+)$")
			if m == nil then file:close () refuse ("the " .. role .. " report's verdict line is malformed") end
			fields = { mode = m, status = s, detail = detail }
		else
			local why
			fields, why = strict_pairs (line)
			if fields == nil then file:close () refuse ("the " .. role .. " report's " .. name .. " record is malformed: " .. why) end
		end
		if kind == "one" and rec [name] ~= nil then file:close () refuse ("the " .. role .. " report holds the single record " .. name .. " twice") end
		rec [name] = rec [name] or {}
		if kind == "one" then rec [name] = fields else rec [name] [#rec [name] + 1] = fields end
		order [#order + 1] = name
		last = name
	end
	file:close ()
	if order [1] ~= "sh-witness-identity" or last ~= "sh-witness-verdict" then refuse ("the " .. role .. " report does not begin with its identity and end with its verdict") end
	local id, verdict_record = rec ["sh-witness-identity"], rec ["sh-witness-verdict"]
	if id.schema ~= "2" or id.mode ~= expected_mode or verdict_record.mode ~= expected_mode or verdict_record.status ~= "PASS" then
		refuse ("the " .. role .. " report is not a successful schema 2 " .. expected_mode .. " report (mode " .. tostring (id.mode) .. ", verdict " .. tostring (verdict_record.status) .. ")")
	end
	if id.profile ~= MODES [expected_mode].profile then refuse ("the " .. role .. " report names profile " .. tostring (id.profile) .. ", not " .. MODES [expected_mode].profile) end
	for key, mine in pairs ({ scenario = scenario_name, seed = tostring (seed), frame_ms = tostring (frame_ms), hours = tostring (hours) }) do
		if id [key] ~= mine then refuse ("the " .. role .. " report is of another setup: " .. key .. "=" .. tostring (id [key]) .. ", this run " .. mine) end
	end
	if rec ["sh-witness-counts"] == nil then refuse ("the " .. role .. " report has no counts record") end
	return rec
end

local reference = {}
if mode == "clearance_control" and args.original ~= nil then
	local rec = read_reference (args.original, "original", "clearance_original")
	local h = rec ["sh-witness-handoff"]
	if h == nil or h.ordinal ~= "1" or rec ["sh-witness-counts"].height_reads ~= "0" then refuse ("the original report holds no first hand-off with the host never asked") end
	reference.original = h
elseif plan.family == "clearance" and plan.intervene then
	local rec = read_reference (args.control, "control", "clearance_control")
	local h, t = rec ["sh-witness-handoff"], rec ["sh-witness-trajectory"]
	if h == nil or t == nil or rec ["sh-witness-original"] == nil or rec ["sh-witness-original"].equal ~= "1" then
		refuse ("the control report is not a PAIRED control with its hand-off and trajectory (an unpaired control makes no parity and unlocks no intervention)")
	end
	if tonumber (t.window) ~= target_window or tonumber (h.window) ~= target_window then refuse ("window=" .. tostring (target_window) .. " is not the control's window " .. tostring (t.window)) end
	reference.control, reference.trajectory = h, t
elseif loading then
	local rec = read_reference (args.expect, "expect", "save")
	local saved, pack = rec ["sh-witness-save"], rec ["sh-witness-pack"]
	if saved == nil or pack == nil or rec ["sh-save-word"] == nil then refuse ("the save report lacks its save, pack or word records") end
	local rows, by_sector = rec ["sh-save-word"], {}
	for index, row in ipairs (rows) do
		local word = strict_patterns (row.word, 1)
		if tonumber (row.i) ~= index - 1 or tonumber (row.x) == nil or tonumber (row.z) == nil or word == nil then refuse ("the save report's word rows are not ordered, whole and well formed (row " .. index .. ")") end
		local key = row.x .. "," .. row.z
		if by_sector [key] ~= nil then refuse ("the save report holds the sector " .. key .. " twice") end
		by_sector [key] = word [1]
	end
	if tonumber (rec ["sh-witness-counts"].pack_reads) ~= #rows or tonumber (pack.unique) ~= #rows then refuse ("the save report's counts do not equal its " .. #rows .. " word rows") end
	if tonumber (saved.bytes) == nil or tonumber (saved.bytes) <= 4 or saved.name == nil then refuse ("the save report's save record is not whole") end
	local base = string.match (args.campaign, "^(.+)%.S%w%w$")
	if base ~= saved.name then refuse ("campaign=" .. args.campaign .. " is not the saved script of the save report's name " .. tostring (saved.name)) end
	if mode == "load" and save_name == saved.name then refuse ("name= must be a NEW saved name for the second pack, not the save report's") end
	local their_seeds = {}
	for _, s in ipairs (rec ["sh-save-seed"] or {}) do
		local word = strict_patterns (s.word, 1)
		local key = tostring (s.sx) .. "," .. tostring (s.sz)
		if word == nil or their_seeds [key] ~= nil or by_sector [key] ~= word [1] then refuse ("the save report's seed records are not unique, well formed and packed with their bits (" .. key .. ")") end
		their_seeds [key] = word [1]
	end
	for key, mine in pairs (seeds) do
		if their_seeds [key] ~= mine.word then refuse ("the seed " .. key .. " of this run is not the save report's") end
	end
	for key, theirs in pairs (their_seeds) do
		if args.seeds ~= nil and args.seeds ~= "" and (seeds [key] == nil or seeds [key].word ~= theirs) then refuse ("the save report's seed " .. key .. " is not among this run's seeds") end
	end
	reference.rows, reference.by_sector = rows, by_sector
end

if type (host.create_exclusive) ~= "function" then refuse ("this host has no create_exclusive: an output that may never overwrite anything cannot be made safely") end
local out, why = host.create_exclusive (out_path)
if not out then refuse ("cannot create " .. out_path .. " exclusively: " .. tostring (why)) end
local function emit (line) assert (out:write (line .. "\n")) end

-- ---------------------------------------------------------------------------------------------------------------- single precision under the accepted reference profile, without a division
-- the nearest-or-truncated single of a finite double in the normal range: TOWARD ZERO, the accepted SSE profile (src/reference_arithmetic.rs; the real clearance float subtraction runs under it), or nil outside the normal range
local function toward_zero_single (x)
	if x ~= x or x == huge or x == -huge then return nil end
	if x == 0 then return 0 end
	local sign = 1
	if x < 0 then sign, x = -1, -x end
	local m, e = math.frexp (x)
	if e < -125 or e > 127 then return nil end
	local r = math.floor (m * 16777216)
	return sign * math.ldexp (r, e - 24)
end
-- the single-precision a - b under that profile, or nil when the double difference is not exact (a double rounding could then differ): Knuth's two-sum error term is zero only for an exact difference
local function subtract_single (a, b)
	local t = -b
	local s = a + t
	local bb = s - a
	local err = (a - (s - bb)) + (t - bb)
	if err ~= 0 then return nil end
	return toward_zero_single (s)
end

-- ---------------------------------------------------------------------------------------------------------------- the host's questions
local PHASE_OF = { [ZERO] = 0, ["0x3f800000"] = 1, ["0x40000000"] = 2, ["0x40400000"] = 3, ["0x40800000"] = 4, ["0x40e00000"] = 7, ["0x41500000"] = 13, ["0x41600000"] = 14, ["0x41880000"] = 17, ["0x41900000"] = 18 }
-- the entity insertions of the roster: a sector-list INSERT (4), a city child INSERT (7), a FIXED_STATE (13) and a CITY_STATE (14) statement
local INSERT_PHASE = { [4] = true, [7] = true, [13] = true, [14] = true }
local function other_pattern (pattern, delta) return string.format ("0x%08x", tonumber (string.sub (pattern, 3), 16) + delta) end
-- the value of an IEEE single given as 0x%08x text: the number (nil for a NaN) and whether it is a NaN; no division
local function decode (text)
	local n = tonumber (string.sub (text, 3), 16)
	local sign = 1
	if n >= 2147483648 then sign, n = -1, n - 2147483648 end
	local exponent = math.floor (n * 0.00000011920928955078125) -- n * 2^-23, exact
	local mantissa = n - exponent * 8388608
	if exponent == 255 then
		if mantissa ~= 0 then return nil, true end
		return sign * huge, false
	end
	if exponent == 0 then return sign * mantissa * math.ldexp (1, -149), false end
	return sign * math.ldexp (8388608 + mantissa, exponent - 150), false
end
-- the sector a question names (facts 12 and 13 hold its whole indices as singles), or nil
local function sector_of (words)
	local x, z = decode (words [13]), decode (words [14])
	if x == nil or z == nil or x == huge or z == huge or x < 0 or z < 0 or x ~= math.floor (x) or z ~= math.floor (z) then return nil end
	return x, z
end

local state = {
	context = "boot", -- boot | frames | save
	frame = 0, abort = nil,
	in_scan = false, scan_questions = 0, scan_reads = 0,
	window = 0, window_open = false, window_reads = nil, window_info = {}, pending = nil, unbound_lookaheads = 0,
	windows_without_handoff = 0, handoff = nil, handoff_count = 0, altered_windows = 0, altered_reads = 0,
	height_reads = 0, restores = {}, restore_order = {}, restore_duplicates = 0, inserts = 0, inserts_before_first_restore = 0, inserts_since_restore = 0, inserts_between_restores = 0,
	save_attempts = nil, pack_final = nil, pack_info = nil, other_during_save = 0, intervened = 0, reply_faults = {}, request_faults = {}, seeded_checked = 0,
}

-- a window whose complete same-caller sequence (look-ahead, then move of the same host id) did not follow directly had no hand-off
local function drop_pending ()
	if state.pending ~= nil then
		state.windows_without_handoff = state.windows_without_handoff + 1
		state.pending = nil
	end
end

local function close_window ()
	if state.window_open then
		state.window_open = false
		drop_pending ()
		state.pending = { window = state.window, reads = state.window_reads, stage = "lookahead" }
	end
end

-- the first problem of a REAL host reply to a height read or a restore against the C bridge's grammar (mb_hostentity.h check_sector_height_reply), or nil: the state, the list of exactly three patterns
-- (no empty or extra segment), no placement name, the sector echoed as asked and, for a restore, the word stored equal to the word stated
local function reply_problem (answer, words, phase)
	if type (answer) ~= "table" then return "the reply is not a table" end
	local wanted = phase == 18 and "restored" or "height"
	if answer.result ~= wanted then return "result " .. tostring (answer.result) .. ", not " .. wanted end
	if answer.name ~= nil and answer.name ~= "" then return "the reply names a placement" end
	local p = strict_patterns (answer.p, 3)
	if p == nil then return "p is not exactly three 0x%08x patterns" end
	if p [1] ~= words [13] or p [2] ~= words [14] then return "the sector is not the one asked about" end
	if phase == 18 and p [3] ~= words [1] then return "the host stored a word that is not the word stated" end
	return nil
end

local function fault_reply (kind, real, words, phase)
	local p = strict_patterns (real.p, 3)
	if kind == "callback" then error ("the host withholds its sector height answer", 0) end
	if kind == "malformed" then return { result = real.result, p = p [1] .. "," .. p [2] } end
	if kind == "state" then return { result = phase == 18 and "height" or "restored", p = real.p } end
	if kind == "sector" then return { result = real.result, p = other_pattern (p [1], 1) .. "," .. p [2] .. "," .. p [3] } end
	if kind == "stale" then return { result = real.result, p = p [1] .. "," .. p [2] .. "," .. other_pattern (p [3], 1) } end
	error ("unknown fault kind " .. tostring (kind), 0)
end
local FAULT_KIND = {
	clearance_withheld = "callback", clearance_malformed = "malformed", clearance_wrong_state = "state", clearance_wrong_sector = "sector",
	save_withheld = "callback", load_withheld = "callback", load_wrong_sector = "sector", load_wrong_state = "state", load_stale_word = "stale",
}

local ALTERED_WORD = "0x47c35000" -- 100000.0

local function note_reply_problem (what, problem) state.reply_faults [#state.reply_faults + 1] = what .. ": " .. problem end
local function note_request_fault (what) state.request_faults [#state.request_faults + 1] = what end

-- the facts of a request as the bridge sends them (16 exact patterns, then nothing for a sector height question), or nil and the problem
local function parse_request (key)
	local head, rest = string.match (key, "^(%S+)(.*)$")
	if head == nil then return nil, "an empty facts text" end
	local words = strict_patterns (head, 16)
	if words == nil then return nil, "the facts are not exactly 16 comma separated 0x%08x patterns" end
	local phase = PHASE_OF [words [12]]
	if phase == 17 or phase == 18 then
		if rest ~= "" then return nil, "a sector height question names no placement or identity" end
		for index = 1, 16 do
			if not (index == 12 or index == 13 or index == 14 or (phase == 18 and index == 1)) and words [index] ~= ZERO then
				return nil, "a sector height question states nothing but the sector, its phase and, for a restore, the word"
			end
		end
	end
	return words, nil, phase
end

local function trajectory_here ()
	return { window = state.window, frame = state.frame, windows_without_handoff = state.windows_without_handoff, scan_reads = state.scan_reads, handoffs = state.handoff_count }
end

local function pack_note (key_xz, x, z, word)
	local attempts = state.save_attempts
	local current = attempts [#attempts]
	if current == nil or current.seen [key_xz] ~= nil then
		current = { order = {}, words = {}, seen = {} }
		attempts [#attempts + 1] = current
	end
	current.seen [key_xz] = true
	current.order [#current.order + 1] = { key = key_xz, x = x, z = z }
	current.words [key_xz] = word
end

-- the host's model provider: every question is delegated unchanged except the ONE eligible intervention of the mode
local function model (op, scene, depth, kind, definition, key)
	local words, phase
	if op == "entity_scan" and type (key) == "string" then
		local problem
		words, problem, phase = parse_request (key)
		if words == nil then
			note_request_fault ("entity_scan request: " .. problem)
			close_window ()
			drop_pending ()
			return host.model (op, scene, depth, kind, definition, key)
		end
	end
	if phase ~= 17 then
		if state.context == "save" then state.other_during_save = state.other_during_save + 1 end
		-- a model question that is not a sector height read closes an open window and ends any open same-caller relation; scans open at phase 0 (BEGIN) and close at their `done_*` reply
		close_window ()
		drop_pending ()
		if phase == 0 or phase == 1 or phase == 2 or phase == 3 then state.in_scan, state.scan_questions = true, state.scan_questions + 1 end
		if phase ~= nil and INSERT_PHASE [phase] then
			state.inserts = state.inserts + 1
			if state.restore_first_index == nil then state.inserts_before_first_restore = state.inserts_before_first_restore + 1 else state.inserts_since_restore = state.inserts_since_restore + 1 end
		end
		if phase == 18 and sector_of (words) ~= nil then
			local x, z = sector_of (words)
			local key_xz = x .. "," .. z
			state.restore_first_index = state.restore_first_index or 1
			-- an insertion between two restores breaks restore-before-insert
			state.inserts_between_restores = state.inserts_between_restores + state.inserts_since_restore
			state.inserts_since_restore = 0
			local real = host.model (op, scene, depth, kind, definition, key)
			local problem = reply_problem (real, words, 18)
			if problem ~= nil then
				-- a malformed positive is the host's own failure, never the configured single fault
				note_reply_problem ("restore of " .. key_xz, problem)
				return real
			end
			if plan.at == "restore" and state.intervened == 0 then
				state.intervened = 1
				state.intervention = "restore " .. key_xz
				return fault_reply (FAULT_KIND [mode], real, words, 18)
			end
			if state.restores [key_xz] ~= nil then state.restore_duplicates = state.restore_duplicates + 1 end
			if state.restores [key_xz] == nil then state.restore_order [#state.restore_order + 1] = key_xz end
			state.restores [key_xz] = { x = x, z = z, stated = words [1], stored = strict_patterns (real.p, 3) [3] }
			return real
		end
		local answer = host.model (op, scene, depth, kind, definition, key)
		if phase ~= nil and phase <= 3 and type (answer) == "table" and (answer.result == "done_none" or answer.result == "done_hit") then state.in_scan = false end
		return answer
	end
	-- a sector height READ
	local x, z = sector_of (words)
	if x == nil then
		note_request_fault ("a sector height question names a sector that is not a whole non-negative index")
		return host.model (op, scene, depth, kind, definition, key)
	end
	local key_xz = x .. "," .. z
	state.height_reads = state.height_reads + 1
	if state.context == "save" then
		local real = host.model (op, scene, depth, kind, definition, key)
		local problem = reply_problem (real, words, 17)
		if problem ~= nil then
			note_reply_problem ("pack read of " .. key_xz, problem)
			return real
		end
		if plan.at == "pack" and state.intervened == 0 then
			state.intervened = 1
			state.intervention = "pack read " .. key_xz
			return fault_reply (FAULT_KIND [mode], real, words, 17)
		end
		pack_note (key_xz, x, z, strict_patterns (real.p, 3) [3])
		return real
	end
	if state.in_scan then
		close_window ()
		drop_pending ()
		state.scan_reads = state.scan_reads + 1
		return host.model (op, scene, depth, kind, definition, key)
	end
	-- outside any scan and any save: the guarded clearance asking (the census guard pins every reader)
	if plan.family ~= "clearance" then return host.model (op, scene, depth, kind, definition, key) end
	if not state.window_open then
		drop_pending ()
		state.window = state.window + 1
		state.window_open, state.window_reads = true, {}
		state.window_info [state.window] = trajectory_here ()
		-- the trajectory of the window about to be intervened on must be the paired control's: never an intervention on another step
		if plan.intervene and state.window == target_window then
			local t, mine = reference.trajectory, state.window_info [state.window]
			for _, key_name in ipairs ({ "window", "frame", "windows_without_handoff", "scan_reads", "handoffs" }) do
				if tostring (mine [key_name]) ~= t [key_name] then
					state.abort = string.format ("the trajectory of window %d differs from the control's (%s: control %s, this run %s): no intervention was made on another step", target_window, key_name, tostring (t [key_name]), tostring (mine [key_name]))
					break
				end
			end
		end
	end
	local real = host.model (op, scene, depth, kind, definition, key)
	local problem = reply_problem (real, words, 17)
	if problem ~= nil then
		note_reply_problem ("clearance read of " .. key_xz .. " in window " .. state.window, problem)
		return real
	end
	local p = strict_patterns (real.p, 3)
	local answered = p [3]
	if plan.intervene and state.window == target_window and state.abort == nil then
		if plan.intervene == "word" then
			state.altered_reads = state.altered_reads + 1
			if state.altered_windows == 0 then state.altered_windows = 1 end
			state.window_reads [#state.window_reads + 1] = { x = x, z = z, real = answered, answered = ALTERED_WORD }
			state.intervened = 1
			state.intervention = "window " .. state.window .. " word"
			return { result = "height", p = p [1] .. "," .. p [2] .. "," .. ALTERED_WORD }
		end
		if state.intervened == 0 then
			state.intervened = 1
			state.intervention = "window " .. state.window .. " first read"
			state.window_reads [#state.window_reads + 1] = { x = x, z = z, real = answered, answered = "fault" }
			return fault_reply (FAULT_KIND [mode], real, words, 17)
		end
	end
	state.window_reads [#state.window_reads + 1] = { x = x, z = z, real = answered, answered = answered }
	return real
end

-- any other host callback closes an open window and ends any open same-caller relation (there is nothing between the reads of one leader step and its look-ahead and move)
local function other_callback (name)
	return function (...)
		close_window ()
		drop_pending ()
		return host [name] (...)
	end
end

local function number_ok (x) return type (x) == "number" and x == x and x ~= huge and x ~= -huge end
local function id_ok (x) return type (x) == "string" and x ~= "" end
local function position_ok (p) return type (p) == "table" and number_ok (p [1]) and number_ok (p [2]) and number_ok (p [3]) end

-- the hand-off evidence of a fixed_wing_move step and the host's answer, complete or the first thing missing
local function handoff_evidence (step, answer)
	if not id_ok (step.host_id) then return nil, "no host id" end
	if not number_ok (step.elevation) then return nil, "no finite elevation" end
	if type (step.fixed_wing_terrain) ~= "table" or not number_ok (step.fixed_wing_terrain.min_height) then return nil, "no finite min_height" end
	if type (step.pose) ~= "table" or not position_ok (step.pose.position) then return nil, "no finite pre pose position" end
	if not (number_ok (step.phase) and step.phase == math.floor (step.phase) and step.phase >= 0 and step.phase <= 3) then return nil, "no phase" end
	if type (answer) ~= "table" or not position_ok (answer.position) then return nil, "the host's answer holds no finite position" end
	return true
end

local function move_vehicle (step)
	close_window ()
	local is_table = type (step) == "table"
	local movement = is_table and step.mode or nil
	local answer = host.move_vehicle (step)
	local pending = state.pending
	if movement == "fixed_wing_lookahead" then
		if not (id_ok (step.host_id) and type (answer) == "table") then
			if is_table and id_ok (step.host_id) then note_reply_problem ("fixed_wing_lookahead of " .. step.host_id, "the host answered no table") end
			drop_pending ()
		elseif plan.original and pending == nil then
			-- the original-solver profile has no reads: the relation starts at the look-ahead itself
			state.pending = { window = 0, reads = {}, stage = "move", host_id = step.host_id }
		elseif pending ~= nil and pending.stage == "lookahead" then
			pending.stage, pending.host_id = "move", step.host_id
		else
			state.unbound_lookaheads = state.unbound_lookaheads + 1
			drop_pending ()
		end
	elseif movement == "fixed_wing_move" and type (step.fixed_wing_terrain) == "table" then
		state.pending = nil
		if pending ~= nil and pending.stage == "move" and pending.host_id == step.host_id then
			local ok, missing = handoff_evidence (step, answer)
			if ok then
				state.handoff_count = state.handoff_count + 1
				local wanted = state.handoff == nil and (plan.original or plan.intervene == nil or (plan.intervene == "word" and pending.window == target_window))
				if wanted then
					local reads = pending.reads
					local last = reads [#reads]
					state.handoff = {
						window = pending.window, ordinal = state.handoff_count, reads = #reads, sector = last and (last.x .. "," .. last.z) or "none", real_word = last and last.real or "none",
						answered_word = last and last.answered or "none", elevation = step.elevation, min_height = step.fixed_wing_terrain.min_height, answered_y = answer.position [2], pre_y = step.pose.position [2],
						phase = step.phase, host_id = (string.gsub (step.host_id, "%s", "_")),
					}
				end
			else
				note_request_fault ("fixed_wing_move hand-off evidence incomplete: " .. missing)
			end
		else
			-- a move with terrain that does not follow a same-caller look-ahead directly (another caller, or another callback in between): no hand-off is credited
			if pending ~= nil then state.windows_without_handoff = state.windows_without_handoff + 1 end
		end
	else
		drop_pending ()
	end
	return answer
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
	if not ok then
		fatal = { marker = engine_marker (err), line = first_line (err), where = where }
	end
	return ok, err
end

local function boot_config (campaign)
	return {
		install_root = root,
		map = args.map or scenario.map,
		terrain = host.terrain_provider (root, args.map or scenario.map), environment = host.env_provider (),
		campaign_directory = "camp01",
		campaign = campaign,
		gunship = "apache",
		seed = seed,
		bind_mobile = plan.family == "clearance" and other_callback ("bind_mobile") or host.bind_mobile,
		move_vehicle = plan.family == "clearance" and move_vehicle or host.move_vehicle,
		observe_loss = plan.family == "clearance" and other_callback ("observe_loss") or host.observe_loss,
		supply_reached = plan.family == "clearance" and other_callback ("supply_reached") or host.supply_reached,
		line_of_sight = plan.family == "clearance" and other_callback ("line_of_sight") or host.line_of_sight,
		model = model,
		sense = plan.family == "clearance" and other_callback ("sense") or host.sense,
		aim = plan.family == "clearance" and other_callback ("aim") or host.aim,
	}
end

-- the host's own state of a sector seeded through ITS provider (never through the campaign): a restore, then the read that proves the provider holds those bits; both replies are checked as completely as the bridge checks them
local function facts_text (phase_pattern, x, z, stated)
	local words = {}
	for index = 1, 16 do words [index] = ZERO end
	words [1] = stated or ZERO
	words [12] = phase_pattern
	words [13] = "0x" .. word_of (x)
	words [14] = "0x" .. word_of (z)
	return table.concat (words, ",")
end

local function seed_provider ()
	for _, key_xz in ipairs (seed_order) do
		local s = seeds [key_xz]
		local restore_text = facts_text ("0x41900000", s.x, s.z, s.word)
		local read_text = facts_text ("0x41880000", s.x, s.z, nil)
		local restored = host.model ("entity_scan", 0, nil, nil, nil, restore_text)
		local problem = reply_problem (restored, strict_patterns (restore_text, 16), 18)
		if problem ~= nil then refuse ("the host's provider did not acknowledge the seed " .. key_xz .. "=" .. s.word .. " after the seeding restore: " .. problem) end
		local read = host.model ("entity_scan", 0, nil, nil, nil, read_text)
		problem = reply_problem (read, strict_patterns (read_text, 16), 17)
		if problem ~= nil then refuse ("the host's provider's read of the seeded sector " .. key_xz .. " is malformed: " .. problem) end
		if strict_patterns (read.p, 3) [3] ~= s.word then refuse ("the host's provider answers a different word for the seeded sector " .. key_xz) end
		state.seeded_checked = state.seeded_checked + 1
	end
end

-- ---------------------------------------------------------------------------------------------------------------- report and verdict
local status, detail = "FAILURE", "no verdict was reached"
local function verdict (s, d) status, detail = s, d end

local function run_clearance ()
	engine = dc.boot (boot_config (args.campaign or scenario.campaign))
	state.context = "frames"
	local frames = math.floor (hours * 3600 * 1000 / frame_ms)
	local ran = 0
	for _ = 1, frames do
		state.frame = state.frame + 1
		local ok = guarded ("frame", function () engine:frame (frame_ms) end)
		ran = ran + 1
		if not ok then break end
		if state.abort ~= nil then break end
		if plan.intervene == nil and state.handoff ~= nil then break end
		if plan.intervene == "word" and state.handoff ~= nil and state.handoff.window == target_window then break end
	end
	state.frames_run = ran
	if fatal == nil then close_window () end
end

local function bits_or (value) local bits = value ~= nil and word_of (value) or nil return bits and ("0x" .. bits) or nil end

-- the verdict of any run starts with the faults that outrank every absence: a request or reply fault of the exact bridge grammar, an aborted intervention
local function fault_verdict ()
	if #state.request_faults > 0 then return verdict ("FAILURE", "the campaign's own request broke the contract: " .. state.request_faults [1]) end
	if #state.reply_faults > 0 then return verdict ("FAILURE", "the host's own answer broke the contract: " .. state.reply_faults [1]) end
	if state.abort ~= nil then return verdict ("FAILURE", mode .. ": " .. state.abort) end
	return false
end

local function clearance_verdict ()
	local h = state.handoff
	if fault_verdict () ~= false then return end
	if plan.intervene == "fault" then
		if fatal == nil then
			if state.intervened == 0 then return verdict ("UNREACHED", string.format ("%s: window %d was never reached (%d windows, %d without a hand-off, %d scan reads): nothing was altered, and no actor, time or window is searched for", mode, target_window, state.window, state.windows_without_handoff, state.scan_reads)) end
			return verdict ("FAILURE", mode .. ": the campaign accepted the faulty answer at " .. state.intervention .. " and went on")
		end
		if state.intervened == 0 then return verdict ("FAILURE", mode .. " stopped at [model-provider:" .. tostring (fatal.marker) .. "] before the eligible read of window " .. target_window .. ": " .. fatal.line) end
		if fatal.marker ~= plan.marker then return verdict ("FAILURE", mode .. " stopped at [model-provider:" .. tostring (fatal.marker) .. "], not at [model-provider:" .. plan.marker .. "]: " .. fatal.line) end
		return verdict ("PASS", string.format ("%s: the engine stopped at [model-provider:%s] after the one intervention at %s", mode, plan.marker, state.intervention))
	end
	if fatal ~= nil then return verdict ("FAILURE", mode .. ": the engine stopped: " .. fatal.line) end
	if plan.original then
		-- the ORIGINAL-solver profile: the campaign consumes its own legacy word and never asks the host for a sector height; a run in which the host WAS asked is not that profile
		if state.height_reads > 0 then return verdict ("FAILURE", string.format ("%s: the host was asked for a sector height %d times: this is not the original-solver profile", mode, state.height_reads)) end
		if h == nil then return verdict ("UNREACHED", string.format ("%s: no complete same-caller hand-off (%d hand-off steps, %d look-aheads unbound, %d frames): nothing is credited, and no actor, time or window is searched for", mode, state.handoff_count, state.unbound_lookaheads, state.frames_run or 0)) end
		if not (number_ok (h.elevation) and number_ok (h.min_height)) then return verdict ("FAILURE", "the hand-off or the terrain elevation is not a finite number") end
		if not (h.min_height >= h.elevation) then return verdict ("FAILURE", string.format ("the hand-off %s is below the terrain elevation %s", tostring (bits_or (h.min_height)), tostring (bits_or (h.elevation)))) end
		return verdict ("PASS", string.format ("%s: hand-off %d, elevation %s, hand-off %s; the legacy word was consumed and the host was never asked for a sector height", mode, h.ordinal, tostring (bits_or (h.elevation)), tostring (bits_or (h.min_height))))
	end
	if h == nil or (plan.intervene == "word" and h.window ~= target_window) then
		return verdict ("UNREACHED", string.format ("%s: no complete same-caller hand-off%s (%d windows, %d without a hand-off, %d look-aheads unbound, %d scan reads, %d frames): nothing is credited, and no actor, time or window is searched for", mode, plan.intervene == "word" and (" in window " .. target_window) or "", state.window, state.windows_without_handoff, state.unbound_lookaheads, state.scan_reads, state.frames_run or 0))
	end
	if plan.intervene == "word" then
		-- the same caller and step as the control, then the accepted round-toward-zero expression
		local c = reference.control
		for _, key in ipairs ({ "host_id", "elevation", "pre_y", "phase" }) do
			local mine = ({ host_id = h.host_id, elevation = bits_or (h.elevation), pre_y = bits_or (h.pre_y), phase = tostring (h.phase) }) [key]
			if mine ~= c [key] then return verdict ("FAILURE", string.format ("%s: the consumed hand-off is not the control's caller and step (%s: control %s, this run %s)", mode, key, tostring (c [key]), tostring (mine))) end
		end
		local expected = subtract_single (100000, h.elevation)
		if expected == nil then return verdict ("FAILURE", mode .. ": the expected hand-off cannot be formed exactly in single precision from elevation " .. tostring (bits_or (h.elevation))) end
		if state.altered_windows ~= 1 or state.intervened ~= 1 then return verdict ("FAILURE", mode .. ": the single eligible intervention was not made exactly once") end
		if word_of (expected) ~= word_of (h.min_height) then
			-- clean non-dominance is a larger hand-off the control already had (an altitude or terrain term dominates); anything else is an unexplained hand-off
			if h.min_height > expected and bits_or (h.min_height) == c.min_height then
				return verdict ("UNREACHED", string.format ("%s: the changed word did not dominate the clearance expression (hand-off %s equals the control's, expected %s): the hand-off is not shown to follow the host's word", mode, tostring (bits_or (h.min_height)), tostring (bits_or (expected))))
			end
			return verdict ("FAILURE", string.format ("%s: the hand-off %s is neither the round-toward-zero value %s of 100000.0 - elevation nor the control's larger term", mode, tostring (bits_or (h.min_height)), tostring (bits_or (expected))))
		end
		state.effect = { handoff = bits_or (h.min_height) ~= c.min_height and "changed" or "unchanged", physical = bits_or (h.answered_y) ~= c.answered_y and "changed" or "unchanged" }
		return verdict ("PASS", string.format ("%s: window %d, the hand-off %s is the round-toward-zero value of 100000.0 - elevation %s, the control's caller", mode, h.window, tostring (bits_or (h.min_height)), tostring (bits_or (h.elevation))))
	end
	-- the control: the hand-off is bounded below by the original expressions over the word the host answered, under the accepted round-toward-zero profile
	local real_value
	if h.real_word ~= "none" then real_value = decode (h.real_word) end
	if real_value == nil or real_value == huge or real_value == -huge then return verdict ("FAILURE", "the consumed word is absent or not finite: the clearance expression was not characterised") end
	local through_word = subtract_single (real_value, h.elevation)
	if through_word == nil then return verdict ("FAILURE", "the lower bound word - elevation cannot be formed exactly in single precision: the clearance expression was not characterised") end
	local floor_value = h.elevation
	if through_word > floor_value then floor_value = through_word end
	if not (h.min_height >= floor_value) then return verdict ("FAILURE", string.format ("the hand-off %s is below max(elevation, word - elevation) = %s", tostring (bits_or (h.min_height)), tostring (bits_or (floor_value)))) end
	if reference.original ~= nil then
		-- the same setup in the ORIGINAL-solver profile (the whole successful clearance_original report): its first hand-off must be this one, field for field
		local mine = { host_id = h.host_id, elevation = bits_or (h.elevation), min_height = bits_or (h.min_height), pre_y = bits_or (h.pre_y), answered_y = bits_or (h.answered_y), phase = tostring (h.phase) }
		local differing = {}
		for _, key in ipairs ({ "host_id", "elevation", "min_height", "pre_y", "answered_y", "phase" }) do
			if reference.original [key] ~= mine [key] then differing [#differing + 1] = key .. " (original " .. tostring (reference.original [key]) .. ", hosted " .. tostring (mine [key]) .. ")" end
		end
		if #differing > 0 then return verdict ("FAILURE", "the hosted first hand-off differs from the original profile's: " .. table.concat (differing, "; ")) end
		state.original_equal = true
	end
	return verdict ("PASS", string.format ("%s (%s): window %d, consumed word %s, elevation %s, hand-off %s >= max(elevation, word - elevation)", mode, state.original_equal and "paired with the original profile" or "UNPAIRED: no parity claim", h.window, tostring (h.real_word), tostring (bits_or (h.elevation)), tostring (bits_or (h.min_height))))
end

-- a saved word's no-lowering relation to the word restored for the sector: held (bits equal), raised (a larger number) or violation
local function relation (restored_bits, final_bits)
	if restored_bits == final_bits then return "held" end
	-- bits that differ: a NaN on either side is a violation (a restored NaN makes every comparison false and stays; nothing becomes one)
	local rv = decode (restored_bits)
	local fv = decode (final_bits)
	if rv == nil or fv == nil then return "violation" end
	if fv > rv then return "raised" end
	if fv == rv then return "held" end -- a zero of the other sign
	return "violation"
end

-- the attempts of one save reconciled: store_session retries pack_session with a larger buffer, each attempt reading every sector again in the same order with the same words. The FINAL attempt is the saved relationship; any
-- attempt that is not a whole repeat of the first (another order, a missing or extra sector, another word) is unsupported repetition and non-pass. Returns the final reads (order, words) and the telemetry, or nil and the problem.
local function reconcile_attempts (attempts)
	if #attempts < 1 then return nil, "the save made no sector read" end
	if #attempts > 64 then return nil, #attempts .. " pack attempts: more than the doubling of any real buffer allows" end
	local first = attempts [1]
	local repeats = 0
	for index = 2, #attempts do
		local a = attempts [index]
		if #a.order ~= #first.order then return nil, string.format ("pack attempt %d read %d sectors, the first read %d: unsupported repetition", index, #a.order, #first.order) end
		for position, item in ipairs (first.order) do
			if a.order [position].key ~= item.key then return nil, string.format ("pack attempt %d read sector %s where the first read %s: unsupported repetition", index, a.order [position].key, item.key) end
			if a.words [item.key] ~= first.words [item.key] then return nil, string.format ("pack attempt %d read %s for sector %s, the first read %s: an inconsistent repeated word", index, a.words [item.key], item.key, first.words [item.key]) end
		end
		repeats = repeats + #a.order
	end
	local final = attempts [#attempts]
	return { order = final.order, words = final.words, info = { attempts = #attempts, unique = #final.order, repeat_reads = repeats } }
end

local function save_once (name)
	state.context = "save"
	state.save_attempts, state.other_during_save = {}, 0
	local ok, text = guarded ("save", function () return engine:sector_height_witness_save (name) end)
	state.context = "frames"
	if ok then state.save_line = first_line (text) end
	if fatal == nil then
		local final, problem = reconcile_attempts (state.save_attempts)
		if final == nil then state.pack_problem = problem else state.pack_final, state.pack_info = final, final.info end
	end
	return ok
end

local function run_session ()
	if saving then
		engine = dc.boot (boot_config (args.campaign or scenario.campaign))
		if type (engine.sector_height_witness_save) ~= "function" then
			refuse ("the module has no sector_height_witness_save: it was built without feature eech-dc/sector-height-witness")
		end
		state.context = "frames"
		for _ = 1, frames_arg do
			local ok = guarded ("frame", function () engine:frame (frame_ms) end)
			if not ok then return end
		end
		seed_provider ()
		save_once (save_name)
		return
	end
	-- a load: the campaign's own boot of the saved script restores; then the campaign saves again
	local ok = guarded ("boot", function () engine = dc.boot (boot_config (args.campaign)) end)
	if not ok then return end
	if type (engine.sector_height_witness_save) ~= "function" then
		refuse ("the module has no sector_height_witness_save: it was built without feature eech-dc/sector-height-witness")
	end
	state.context = "frames"
	if mode == "load" then save_once (save_name) end
end

local function final_read (key_xz)
	local final = state.pack_final
	if final == nil then return nil end
	return final.words [key_xz]
end

local function session_verdict ()
	if fault_verdict () ~= false then return end
	if plan.intervene == "fault" then
		if fatal == nil then
			if state.intervened == 0 then return verdict ("UNREACHED", mode .. ": no " .. plan.at .. " reached: nothing was altered") end
			return verdict ("FAILURE", mode .. ": the campaign accepted the faulty answer at " .. state.intervention .. " and went on")
		end
		if state.intervened == 0 then return verdict ("FAILURE", mode .. " stopped at [model-provider:" .. tostring (fatal.marker) .. "] before the eligible " .. plan.at .. ": " .. fatal.line) end
		if fatal.marker ~= plan.marker then return verdict ("FAILURE", mode .. " stopped at [model-provider:" .. tostring (fatal.marker) .. "], not at [model-provider:" .. plan.marker .. "]: " .. fatal.line) end
		return verdict ("PASS", string.format ("%s: the campaign stopped at [model-provider:%s] after the one intervention at %s", mode, plan.marker, state.intervention))
	end
	if fatal ~= nil then return verdict ("FAILURE", mode .. ": the engine stopped: " .. fatal.line) end
	if state.save_line == nil then return verdict ("FAILURE", mode .. ": the campaign did not report a save") end
	local name, bytes, version = string.match (state.save_line, "^sh%-witness%-save name=(%S+) bytes=(%d+) version=(%-?%d+)$")
	if name == nil or name ~= save_name or tonumber (bytes) <= 4 then return verdict ("FAILURE", "the save report is not whole: " .. state.save_line) end
	if state.pack_problem ~= nil then return verdict ("FAILURE", "the pack attempts of the save do not reconcile: " .. state.pack_problem) end
	local final = state.pack_final
	if mode == "save" then
		for _, key_xz in ipairs (seed_order) do
			local word = final_read (key_xz)
			if word == nil then return verdict ("FAILURE", "the seeded sector " .. key_xz .. " was never packed: not a sector of this map's session") end
			if word ~= seeds [key_xz].word then return verdict ("FAILURE", "the seeded sector " .. key_xz .. " was packed as " .. word .. ", not " .. seeds [key_xz].word) end
		end
		return verdict ("PASS", string.format ("save: %d sectors packed in %d attempt(s), %d seeded sectors packed with their seeds' bits, %s", final.info.unique, final.info.attempts, #seed_order, state.save_line))
	end
	-- the load
	local packed = reference.by_sector
	if #state.restore_order < 1 then return verdict ("UNREACHED", "load: the boot made no restore: the saved script was not loaded as a saved campaign") end
	if state.restore_duplicates > 0 then return verdict ("FAILURE", "a sector was restored more than once (" .. state.restore_duplicates .. ")") end
	if #state.restore_order ~= #reference.rows then return verdict ("FAILURE", string.format ("the load restored %d sectors, the save packed %d", #state.restore_order, #reference.rows)) end
	for _, row in ipairs (reference.rows) do
		local key_xz = row.x .. "," .. row.z
		local restored = state.restores [key_xz]
		if restored == nil then return verdict ("FAILURE", "the sector " .. key_xz .. " was packed but never restored") end
		if restored.stated ~= packed [key_xz] then return verdict ("FAILURE", string.format ("the sector %s was restored as %s, the save packed %s", key_xz, restored.stated, packed [key_xz])) end
		if restored.stored ~= restored.stated then return verdict ("FAILURE", "the host did not acknowledge the bits stated for " .. key_xz) end
	end
	if state.inserts_before_first_restore > 0 then return verdict ("FAILURE", string.format ("%d entity insertions came before the first restore: restore-before-insert is not shown", state.inserts_before_first_restore)) end
	if state.inserts_between_restores > 0 then return verdict ("FAILURE", string.format ("%d entity insertions came between two restores: restore-before-insert is not shown", state.inserts_between_restores)) end
	if state.inserts_since_restore < 1 then return verdict ("UNREACHED", "load: no entity insertion followed the restores: restore-before-insert and no-lowering are not exercised") end
	for _, row in ipairs (reference.rows) do
		local key_xz = row.x .. "," .. row.z
		local word = final_read (key_xz)
		if word == nil then return verdict ("FAILURE", "the second pack never read the sector " .. key_xz) end
		if relation (state.restores [key_xz].stated, word) == "violation" then return verdict ("FAILURE", string.format ("the sector %s was restored as %s and read back as %s: LOWERED or a NaN changed", key_xz, state.restores [key_xz].stated, word)) end
	end
	if final.info.unique ~= #reference.rows then return verdict ("FAILURE", string.format ("the second pack read %d sectors, the first %d", final.info.unique, #reference.rows)) end
	return verdict ("PASS", string.format ("load: %d sectors restored exactly as saved and acknowledged, all before the first entity insertion (%d after the last restore), none lowered, second pack in %d attempt(s)", #state.restore_order, state.inserts_since_restore, final.info.attempts))
end

local function report ()
	emit (string.format ("sh-witness-identity schema=2 mode=%s profile=%s scenario=%s seed=%s frame_ms=%s hours=%s window=%s campaign=%s name=%s", mode, plan.profile, scenario_name, tostring (seed), tostring (frame_ms), tostring (hours), target_window and tostring (target_window) or "none", args.campaign or "none", save_name or "none"))
	if plan.family == "clearance" then
		emit (string.format ("sh-witness-counts windows=%d windows_without_handoff=%d scan_reads=%d height_reads=%d altered_windows=%d altered_reads=%d handoffs=%d unbound_lookaheads=%d frames=%s", state.window, state.windows_without_handoff, state.scan_reads, state.height_reads, state.altered_windows, state.altered_reads, state.handoff_count, state.unbound_lookaheads, tostring (state.frames_run)))
		local h = state.handoff
		if h ~= nil then
			emit (string.format ("sh-witness-handoff ordinal=%d window=%d reads=%d sector=%s real_word=%s answered_word=%s host_id=%s elevation=%s min_height=%s pre_y=%s answered_y=%s phase=%s", h.ordinal, h.window, h.reads, h.sector, h.real_word, h.answered_word, h.host_id, tostring (bits_or (h.elevation)), tostring (bits_or (h.min_height)), tostring (bits_or (h.pre_y)), tostring (bits_or (h.answered_y)), tostring (h.phase)))
			local t = state.window_info [h.window]
			if plan.profile == "hosted" and plan.intervene == nil and t ~= nil then
				emit (string.format ("sh-witness-trajectory window=%d frame=%d windows_without_handoff=%d scan_reads=%d handoffs=%d", t.window, t.frame, t.windows_without_handoff, t.scan_reads, t.handoffs))
			end
			if state.original_equal then emit ("sh-witness-original equal=1 fields=host_id,elevation,min_height,pre_y,answered_y,phase") end
			if state.effect then emit (string.format ("sh-witness-effect handoff=%s physical=%s", state.effect.handoff, state.effect.physical)) end
		end
	else
		local pack = state.pack_info
		emit (string.format ("sh-witness-counts height_reads=%d pack_reads=%d pack_other_questions=%d restores=%d restore_duplicates=%d inserts=%d inserts_before_first_restore=%d inserts_between_restores=%d inserts_after_last_restore=%d seeds_checked=%d", state.height_reads, pack and pack.unique or 0, state.other_during_save, #state.restore_order, state.restore_duplicates, state.inserts, state.inserts_before_first_restore, state.inserts_between_restores, state.inserts_since_restore, state.seeded_checked))
		if state.save_line then emit (state.save_line) end
		if pack then emit (string.format ("sh-witness-pack attempts=%d unique=%d repeat_reads=%d consistent=1", pack.attempts, pack.unique, pack.repeat_reads)) end
		for _, key_xz in ipairs (seed_order) do emit (string.format ("sh-save-seed sx=%s sz=%s word=%s", seeds [key_xz].x, seeds [key_xz].z, seeds [key_xz].word)) end
		if mode == "save" and state.pack_final then
			for index, item in ipairs (state.pack_final.order) do
				emit (string.format ("sh-save-word i=%d x=%d z=%d word=%s", index - 1, item.x, item.z, state.pack_final.words [item.key]))
			end
		end
		if loading then
			for index, key_xz in ipairs (state.restore_order) do
				local row = state.restores [key_xz]
				local word = final_read (key_xz)
				emit (string.format ("sh-load-restore i=%d x=%d z=%d stated=%s stored=%s final=%s relation=%s", index - 1, row.x, row.z, row.stated, tostring (row.stored), word or "none", word and relation (row.stated, word) or "none"))
			end
		end
	end
	if fatal ~= nil then emit (string.format ("sh-witness-fatal where=%s marker=%s line=%s", fatal.where, tostring (fatal.marker), string.gsub (fatal.line, "%s+", " "))) end
	for _, fault in ipairs (state.reply_faults) do emit ("sh-witness-reply-fault " .. string.gsub (fault, "%s+", " ")) end
	for _, fault in ipairs (state.request_faults) do emit ("sh-witness-request-fault " .. string.gsub (fault, "%s+", " ")) end
	emit (string.format ("sh-witness-verdict mode=%s status=%s detail=%s", mode, status, string.gsub (detail, "%s+", " ")))
end

local ran, failure = xpcall (function ()
	dc.prepare_installation (root)
	host.use_reference_arithmetic ()
	if plan.family == "clearance" then run_clearance () else run_session () end
	if plan.family == "clearance" then clearance_verdict () else session_verdict () end
end, function (e) return e end)
if not ran then verdict ("FAILURE", "the witness stopped: " .. first_line (failure)) end
local reported, report_failure = pcall (report)
local closed, close_error = out:close ()
if not reported then error (NAME .. ": the report could not be written: " .. tostring (report_failure), 0) end
if not closed then error (NAME .. ": OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error), 0) end
if status ~= "PASS" then error (NAME .. ": " .. status .. ": " .. detail, 0) end
host.log (NAME .. ": " .. detail)
