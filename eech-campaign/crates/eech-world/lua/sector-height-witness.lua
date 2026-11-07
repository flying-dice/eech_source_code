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
-- with nothing between them. The hand-off is the next `fixed_wing_move` step carrying `fixed_wing_terrain` that follows a `fixed_wing_lookahead` step: its `min_height` is the campaign's own result of the ORIGINAL expressions
-- max(terrain elevation, word - terrain elevation, waypoint altitude), and `elevation` is that terrain elevation. The host answers the step with its physical result (the reference host's climb arithmetic).
--
--   clearance_original         the same declared window in the ORIGINAL-solver profile (a build that keeps the original entity solver: `eech-dc/collision-fixture`, no witness feature): the campaign consumes its own legacy word and the host is
--                              never asked for a sector height (a run in which it was is FAILURE: not that profile). Runs until the FIRST hand-off and reports it (the word itself is not observable in this profile): the characterisation FIRST.
--   clearance_control          nothing is altered; with `original=<the clearance_original report>` its first hand-off must equal the original profile's, field for field (host id, elevation, min_height, pre and answered height, phase). Runs until the FIRST window that is followed by a hand-off, then stops: it reports that window's ordinal (`window=`), the consumed word, the terrain elevation, the hand-off and the host's
--                              answered height. PASS needs one such window and `min_height >= max(elevation, word - elevation)` over single-precision values; UNREACHED is a window with no hand-off, none at all, or none inside `hours`.
--   clearance_changed_word     `window=<N>` (from the control, identical campaign, same seed and hours): EVERY word the campaign reads in window N is answered 100000.0 (0x47c35000: a valid finite word that dominates the
--                              expression). PASS needs the hand-off of window N to equal the single-precision value of 100000.0 - elevation exactly (the changed Host input reaching the campaign's own hand-off), and, when `control_min=` and
--                              `control_answer_y=` (bit patterns from the control) are given, the hand-off and the host's answered height to differ from the control's (`physical=changed|unchanged`; unchanged is stated, not hidden).
--   clearance_withheld         the host raises at the FIRST read of window N:                                  [model-provider:entity_scan.callback]
--   clearance_malformed        two patterns instead of three:                                                  [model-provider:entity_scan.answer]
--   clearance_wrong_state      `restored` for a read:                                                          [model-provider:entity_scan.state]
--   clearance_wrong_sector     the sector echoed one unit away:                                                [model-provider:entity_scan.sector_height]
--   A refusal mode is credited ONLY with the one eligible intervention actually made AND the engine stopping at exactly that marker after it; any fatal before it, a run that ends without a fatal, or another marker is a FAILURE; a
--   window never reached is UNREACHED.
--
-- M6, the native save and the campaign's own load (needs a DLL built with `eech-dc/sector-height-witness`; ordinary shipping exports unchanged). The save is the campaign's own `store_session` (uisession.c) over the live session,
-- called by the fixture method: it writes the saved script and the saved data (the session version, then pack_session PACK_MODE_SERVER_SESSION), and packing a sector asks the host for its word. The load is the campaign's own too: a
-- LATER BOOT of the saved script (parsgen.c FILE_TAG_SAVED_CAMPAIGN -> unpack_session), whose sector unpack states each saved word RAW to the host (phase 18), the host holding exactly those bits. No method of this driver packs,
-- unpacks or restores for the campaign. Seeds are the host's explicit raw HISTORIES (an INPUT): before the save the driver states them to its OWN provider as a restore (the host's world holding a history), never to the campaign.
--
--   save                       args: name=<1 to 24 of A-Z a-z 0-9 _ ->, seeds=<sx:sz:0xWORD,...> (optional; raw words: negative, infinite and quiet NaN are valid history, a SIGNALLING NaN is refused: unmeasured), frames (default 0).
--                              Boots, seeds, makes the campaign save, and records every word the pack asked for (`sh-save-word`). PASS needs the save line, every seeded sector packed with its seed's bits and at least one sector.
--   save_withheld              the host raises at the FIRST pack read of the save: [model-provider:entity_scan.callback] (a fault after the eligible read; a fatal before it is a FAILURE)
--   load                       args: campaign=<the saved script's file name>, expect=<the save run's report>, name=<a NEW saved name for the second pack>, seeds=<as in the save run> (optional). Boots the saved script (the
--                              campaign's load) and records every restore the campaign made. PASS needs: every restore stated exactly the word the save packed for that sector (and no extra, none missing), the host acknowledged
--                              those bits, every restore BEFORE the first entity insertion (restore-before-insert), then the campaign saves again (`name`) and no sector's final word is LOWER than the word restored for it (a NaN
--                              unchanged, an infinity unchanged unless raised, a negative history raised only upward).
--   load_withheld / load_wrong_sector / load_wrong_state / load_stale_word
--                              ONE fault at the FIRST restore, the campaign's own boot stopping at the named marker: callback / entity_scan.sector_height (a restore echoed for another sector) / entity_scan.state (`height` for a
--                              restore) / entity_scan.sector_height (the host stored a word that is not the one stated). Positive-first: each is the single fault of the same boot as `load`.
--
-- host.args: root, out (the report; created EXCLUSIVELY first), mode, scenario (lebanon_retail default, georgia_retail), map, campaign (a load's saved script), hours (M5 default 3: the accepted regression-pack window, 3
-- simulated hours, 100 ms frames, seed 1; named here, before any launch), frame_ms (100), seed (1), window, control_min, control_answer_y, name, seeds, expect, frames.
-- NOTHING IN THIS FILE DIVIDES BY A VALUE THAT CAN BE ZERO (a divide by zero is a hardware trap in the native host that pcall cannot catch): the one division is the frame count by `frame_ms`, asserted positive. The float
-- helpers below are the same block as entity-authority.lua's (tests/sector_height_witness_driver.rs compares them); the single-precision rounding and decoding use frexp/ldexp and multiplication only.

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
local PATTERN = "^0x%x%x%x%x%x%x%x%x$"

local MODES = {
	clearance_original = { family = "clearance", original = true },
	clearance_control = { family = "clearance" },
	clearance_changed_word = { family = "clearance", intervene = "word" },
	clearance_withheld = { family = "clearance", intervene = "fault", marker = "entity_scan.callback" },
	clearance_malformed = { family = "clearance", intervene = "fault", marker = "entity_scan.answer" },
	clearance_wrong_state = { family = "clearance", intervene = "fault", marker = "entity_scan.state" },
	clearance_wrong_sector = { family = "clearance", intervene = "fault", marker = "entity_scan.sector_height" },
	save = { family = "session" },
	save_withheld = { family = "session", intervene = "fault", marker = "entity_scan.callback", at = "pack" },
	load = { family = "session" },
	load_withheld = { family = "session", intervene = "fault", marker = "entity_scan.callback", at = "restore" },
	load_wrong_sector = { family = "session", intervene = "fault", marker = "entity_scan.sector_height", at = "restore" },
	load_wrong_state = { family = "session", intervene = "fault", marker = "entity_scan.state", at = "restore" },
	load_stale_word = { family = "session", intervene = "fault", marker = "entity_scan.sector_height", at = "restore" },
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
if args.original ~= nil and (type (args.original) ~= "string" or args.original == "") then refuse ("original=<the clearance_original report>") end

-- the host's raw histories (M6): sx:sz:0xWORD, comma separated; a SIGNALLING NaN is refused (unmeasured), every other word is valid history
local function bits_of (text) return tonumber (string.sub (text, 3), 16) end
local function signalling_nan (text)
	local n = bits_of (text)
	return (n >= 0x7f800001 and n < 0x7fc00000) or (n >= 0xff800001 and n < 0xffc00000)
end
local seeds, seed_order = {}, {}
if args.seeds ~= nil and args.seeds ~= "" then
	if plan.family ~= "session" then refuse ("seeds= is only for a session mode") end
	for item in string.gmatch (args.seeds, "[^,]+") do
		local x, z, word = string.match (item, "^(%d+):(%d+):(0x%x%x%x%x%x%x%x%x)$")
		if x == nil then refuse ("a seed is sx:sz:0xWORD with a whole sector index and eight hex digits, got \"" .. item .. "\"") end
		if signalling_nan (word) then refuse ("the seed " .. item .. " is a signalling NaN: unmeasured, never seeded") end
		local key = x .. "," .. z
		if seeds [key] ~= nil then refuse ("the sector " .. key .. " is seeded twice") end
		seeds [key] = { x = tonumber (x), z = tonumber (z), word = string.lower (word) }
		seed_order [#seed_order + 1] = key
	end
end
local save_name = args.name
if plan.family == "session" and (mode == "save" or mode == "save_withheld" or mode == "load") then
	if type (save_name) ~= "string" or not string.match (save_name, "^[%w_%-]+$") or #save_name > 24 then refuse ("name=<1 to 24 of A-Z a-z 0-9 _ -> is required") end
end
if loading then
	if type (args.campaign) ~= "string" or args.campaign == "" then refuse ("a load needs campaign=<the saved script's file name>") end
	if mode == "load" and (type (args.expect) ~= "string" or args.expect == "") then refuse ("mode load needs expect=<the save run's report>") end
end
local frames_arg = tonumber (args.frames or "0")
assert (frames_arg and frames_arg >= 0 and frames_arg == math.floor (frames_arg), "frames must be a whole number")

if type (host.create_exclusive) ~= "function" then refuse ("this host has no create_exclusive: an output that may never overwrite anything cannot be made safely") end
local out, why = host.create_exclusive (out_path)
if not out then refuse ("cannot create " .. out_path .. " exclusively: " .. tostring (why)) end
local function emit (line) assert (out:write (line .. "\n")) end

-- ---------------------------------------------------------------------------------------------------------------- single precision without a division
-- the nearest IEEE single (ties to even) of a finite double in the normal range, or nil outside it; used only for the guarded clearance's expected expression
local function to_single (x)
	if x ~= x or x == huge or x == -huge then return nil end
	if x == 0 then return 0 end
	local sign = 1
	if x < 0 then sign, x = -1, -x end
	local m, e = math.frexp (x)
	if e < -125 or e > 127 then return nil end
	local scaled = m * 16777216
	local r = math.floor (scaled)
	local rem = scaled - r
	if rem > 0.5 or (rem == 0.5 and r % 2 == 1) then r = r + 1 end
	return sign * math.ldexp (r, e - 24)
end

-- ---------------------------------------------------------------------------------------------------------------- the host's questions
local function words_of (text)
	if type (text) ~= "string" then return nil end
	local head = string.match (text, "^(%S+)")
	if head == nil then return nil end
	local words = {}
	for token in string.gmatch (head, "[^,]+") do
		if not string.match (token, PATTERN) then return nil end
		words [#words + 1] = string.lower (token)
	end
	if #words ~= 16 then return nil end
	return words
end
local PHASE_OF = { [ZERO] = 0, ["0x3f800000"] = 1, ["0x40000000"] = 2, ["0x40400000"] = 3, ["0x40800000"] = 4, ["0x40e00000"] = 7, ["0x41500000"] = 13, ["0x41600000"] = 14, ["0x41880000"] = 17, ["0x41900000"] = 18 }
-- the entity insertions of the roster: a sector-list INSERT (4), a city child INSERT (7), a FIXED_STATE (13) and a CITY_STATE (14) statement
local INSERT_PHASE = { [4] = true, [7] = true, [13] = true, [14] = true }
local function p_words (text)
	if type (text) ~= "string" then return nil end
	local t = {}
	for token in string.gmatch (text, "[^,]+") do t [#t + 1] = token end
	return t
end
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
	in_scan = false, scan_questions = 0, scan_reads = 0,
	window = 0, window_open = false, window_reads = nil, last_closed = nil, pending = nil,
	windows_without_handoff = 0, handoff = nil, handoff_count = 0, lookahead_pending = false, altered_windows = 0, altered_reads = 0,
	height_reads = 0, restores = {}, restore_order = {}, restore_duplicates = 0, inserts = 0, inserts_before_first_restore = 0, inserts_since_restore = 0, inserts_between_restores = 0,
	pack_reads = {}, pack_order = {}, pack_duplicates = 0, other_during_save = 0, intervened = 0, reply_faults = {}, seeded_checked = 0,
}

local function close_window ()
	if state.window_open then
		state.window_open = false
		-- a window whose hand-off never came before the next window closed had none
		if state.pending ~= nil then state.windows_without_handoff = state.windows_without_handoff + 1 end
		state.pending = { window = state.window, reads = state.window_reads, lookahead = false }
		state.last_closed = state.window
	end
end

-- the first problem of a REAL host reply to a height read or a restore, or nil (the host's own answers are checked, never trusted blindly)
local function reply_problem (answer, words, phase)
	if type (answer) ~= "table" then return "the reply is not a table" end
	local wanted = phase == 18 and "restored" or "height"
	if answer.result ~= wanted then return "result " .. tostring (answer.result) .. ", not " .. wanted end
	local p = p_words (answer.p)
	if p == nil or #p ~= 3 then return "p is not three patterns" end
	for index = 1, 3 do if not string.match (p [index], PATTERN) then return "p holds a token that is not a 0x%08x pattern" end end
	if string.lower (p [1]) ~= words [13] or string.lower (p [2]) ~= words [14] then return "the sector is not the one asked about" end
	if phase == 18 and string.lower (p [3]) ~= words [1] then return "the host stored a word that is not the word stated" end
	return nil
end

local function fault_reply (kind, real, words, phase)
	local p = p_words (real.p)
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

local function note_reply_problem (what, problem)
	state.reply_faults [#state.reply_faults + 1] = what .. ": " .. problem
end

-- the host's model provider: every question is delegated unchanged except the ONE eligible intervention of the mode
local function model (op, scene, depth, kind, definition, key)
	local words = op == "entity_scan" and words_of (key) or nil
	local phase = words and PHASE_OF [words [12]] or nil
	if phase ~= 17 then
		if state.context == "save" then state.other_during_save = state.other_during_save + 1 end
		-- a model question that is not a sector height read closes an open window; scans open at phase 0 (BEGIN) and close at their `done_*` reply
		close_window ()
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
			local real
			if plan.at == "restore" and state.intervened == 0 then
				real = host.model (op, scene, depth, kind, definition, key)
				local problem = reply_problem (real, words, 18)
				if problem ~= nil then note_reply_problem ("restore of " .. key_xz, problem) end
				state.intervened = 1
				state.intervention = "restore " .. key_xz
				return fault_reply (FAULT_KIND [mode], real, words, 18)
			end
			real = host.model (op, scene, depth, kind, definition, key)
			local problem = reply_problem (real, words, 18)
			if problem ~= nil then note_reply_problem ("restore of " .. key_xz, problem) end
			if state.restores [key_xz] ~= nil then state.restore_duplicates = state.restore_duplicates + 1 end
			if state.restores [key_xz] == nil then state.restore_order [#state.restore_order + 1] = key_xz end
			state.restores [key_xz] = { x = x, z = z, stated = words [1], stored = p_words (real.p) and string.lower (p_words (real.p) [3] or "") or "" }
			return real
		end
		local answer = host.model (op, scene, depth, kind, definition, key)
		if phase ~= nil and phase <= 3 and type (answer) == "table" and (answer.result == "done_none" or answer.result == "done_hit") then state.in_scan = false end
		return answer
	end
	-- a sector height READ
	local x, z = sector_of (words)
	if x == nil then return host.model (op, scene, depth, kind, definition, key) end
	local key_xz = x .. "," .. z
	state.height_reads = state.height_reads + 1
	if state.context == "save" then
		local real = host.model (op, scene, depth, kind, definition, key)
		local problem = reply_problem (real, words, 17)
		if problem ~= nil then note_reply_problem ("pack read of " .. key_xz, problem) end
		if plan.at == "pack" and state.intervened == 0 then
			state.intervened = 1
			state.intervention = "pack read " .. key_xz
			return fault_reply (FAULT_KIND [mode], real, words, 17)
		end
		if state.pack_reads [key_xz] ~= nil then state.pack_duplicates = state.pack_duplicates + 1 end
		if state.pack_reads [key_xz] == nil then state.pack_order [#state.pack_order + 1] = key_xz end
		local p = p_words (real.p)
		state.pack_reads [key_xz] = { x = x, z = z, word = p and string.lower (p [3] or "") or "" }
		return real
	end
	if state.in_scan then
		state.scan_reads = state.scan_reads + 1
		return host.model (op, scene, depth, kind, definition, key)
	end
	-- outside any scan and any save: the guarded clearance asking (the census guard pins every reader)
	if plan.family ~= "clearance" then return host.model (op, scene, depth, kind, definition, key) end
	if not state.window_open then
		state.window = state.window + 1
		state.window_open, state.window_reads = true, {}
	end
	local real = host.model (op, scene, depth, kind, definition, key)
	local problem = reply_problem (real, words, 17)
	if problem ~= nil then note_reply_problem ("clearance read of " .. key_xz .. " in window " .. state.window, problem) end
	local p = p_words (real.p)
	local answered = p and string.lower (p [3] or "") or ""
	if plan.intervene and state.window == target_window and problem == nil then
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

-- any other host callback closes an open window (there is nothing between the two reads of one leader step)
local function other_callback (name)
	return function (...)
		close_window ()
		return host [name] (...)
	end
end

local function number_ok (x) return type (x) == "number" and x == x and x ~= huge and x ~= -huge end

local function move_vehicle (step)
	close_window ()
	local pending = state.pending
	local is_table = type (step) == "table"
	local movement = is_table and step.mode or nil
	-- a hand-off is a fixed_wing_move carrying fixed_wing_terrain right after a fixed_wing_lookahead (the order the source asks them in)
	local terrain_move = movement == "fixed_wing_move" and type (step.fixed_wing_terrain) == "table" and state.lookahead_pending
	if movement == "fixed_wing_lookahead" then
		state.lookahead_pending = true
	elseif movement == "fixed_wing_move" then
		state.lookahead_pending = false
	end
	if terrain_move then state.handoff_count = state.handoff_count + 1 end
	if pending ~= nil and is_table then
		if movement == "fixed_wing_lookahead" then
			pending.lookahead = true
		elseif movement == "fixed_wing_move" then
			state.pending = nil
			if not (pending.lookahead and terrain_move) then state.windows_without_handoff = state.windows_without_handoff + 1 end
		else
			state.pending = nil
			state.windows_without_handoff = state.windows_without_handoff + 1
		end
	end
	local wanted = false
	if terrain_move and state.handoff == nil then
		if plan.original then
			wanted = true
		elseif pending ~= nil and pending.lookahead and (plan.intervene == nil or (plan.intervene == "word" and pending.window == target_window)) then
			wanted = true
		end
	end
	local answer = host.move_vehicle (step)
	if wanted then
		local reads = pending ~= nil and pending.reads or {}
		local last = reads [#reads]
		local y = type (answer) == "table" and type (answer.position) == "table" and answer.position [2] or nil
		local pre_y = type (step.pose) == "table" and type (step.pose.position) == "table" and step.pose.position [2] or nil
		state.handoff = {
			window = pending ~= nil and pending.window or 0, ordinal = state.handoff_count, reads = #reads, sector = last and (last.x .. "," .. last.z) or "none", real_word = last and last.real or "none",
			answered_word = last and last.answered or "none", elevation = step.elevation, min_height = step.fixed_wing_terrain.min_height, answered_y = y, pre_y = pre_y, phase = step.phase,
			host_id = type (step.host_id) == "string" and (string.gsub (step.host_id, "%s", "_")) or "none",
		}
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

-- the host's own state of a sector seeded through ITS provider (never through the campaign): a restore, then the read that proves the provider holds those bits
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
		local p = type (restored) == "table" and p_words (restored.p) or nil
		if type (restored) ~= "table" or restored.result ~= "restored" or p == nil or string.lower (p [3] or "") ~= s.word then
			refuse ("the host's provider did not hold the seed " .. key_xz .. "=" .. s.word .. " after the seeding restore")
		end
		local read = host.model ("entity_scan", 0, nil, nil, nil, read_text)
		local q = type (read) == "table" and p_words (read.p) or nil
		if type (read) ~= "table" or read.result ~= "height" or q == nil or string.lower (q [3] or "") ~= s.word then
			refuse ("the host's provider answers a different word for the seeded sector " .. key_xz)
		end
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
		local ok = guarded ("frame", function () engine:frame (frame_ms) end)
		ran = ran + 1
		if not ok then break end
		if plan.intervene == nil and state.handoff ~= nil then break end
		if plan.intervene == "word" and state.handoff ~= nil and state.handoff.window == target_window then break end
	end
	state.frames_run = ran
	close_window ()
end

local function bits_or (value) local bits = value ~= nil and word_of (value) or nil return bits and ("0x" .. bits) or nil end

local function clearance_verdict ()
	local h = state.handoff
	if #state.reply_faults > 0 then return verdict ("FAILURE", "the host's own answer broke the contract: " .. state.reply_faults [1]) end
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
		if h == nil then return verdict ("UNREACHED", string.format ("%s: no hand-off (%d hand-off steps in %d frames): nothing is credited, and no actor, time or window is searched for", mode, state.handoff_count, state.frames_run or 0)) end
		if not (number_ok (h.elevation) and number_ok (h.min_height)) then return verdict ("FAILURE", "the hand-off or the terrain elevation is not a finite number") end
		if not (h.min_height >= h.elevation) then return verdict ("FAILURE", string.format ("the hand-off %s is below the terrain elevation %s", tostring (bits_or (h.min_height)), tostring (bits_or (h.elevation)))) end
		return verdict ("PASS", string.format ("%s: hand-off %d, elevation %s, hand-off %s; the legacy word was consumed and the host was never asked for a sector height", mode, h.ordinal, tostring (bits_or (h.elevation)), tostring (bits_or (h.min_height))))
	end
	if h == nil or (plan.intervene == "word" and h.window ~= target_window) then
		return verdict ("UNREACHED", string.format ("%s: no hand-off%s (%d windows, %d without a hand-off, %d scan reads, %d frames): nothing is credited, and no actor, time or window is searched for", mode, plan.intervene == "word" and (" in window " .. target_window) or "", state.window, state.windows_without_handoff, state.scan_reads, state.frames_run or 0))
	end
	if not (number_ok (h.elevation) and number_ok (h.min_height)) then return verdict ("FAILURE", "the hand-off or the terrain elevation is not a finite number") end
	if plan.intervene == "word" then
		local expected = to_single (100000 - h.elevation)
		if expected == nil then return verdict ("FAILURE", "the expected hand-off cannot be formed in single precision") end
		if word_of (expected) ~= word_of (h.min_height) then
			return verdict ("UNREACHED", string.format ("%s: the changed word did not dominate the clearance expression (hand-off %s, expected %s): the hand-off is not shown to follow the host's word", mode, tostring (word_of (h.min_height)), tostring (word_of (expected))))
		end
		if state.altered_windows ~= 1 or state.intervened ~= 1 then return verdict ("FAILURE", mode .. ": the single eligible intervention was not made exactly once") end
		return verdict ("PASS", string.format ("%s: window %d, the hand-off %s is the single-precision value of 100000.0 - elevation %s", mode, h.window, tostring (word_of (h.min_height)), tostring (word_of (h.elevation))))
	end
	-- the control: the hand-off is bounded below by the original expressions over the word the host answered
	local real_value
	if h.real_word ~= "none" then real_value = decode (h.real_word) end
	if real_value == nil or real_value == huge or real_value == -huge then return verdict ("FAILURE", "the consumed word is absent or not finite: the clearance expression was not characterised") end
	local through_word = to_single (real_value - h.elevation)
	local floor_value = h.elevation
	if through_word ~= nil and through_word > floor_value then floor_value = through_word end
	if not (h.min_height >= floor_value) then return verdict ("FAILURE", string.format ("the hand-off %s is below max(elevation, word - elevation) = %s", tostring (word_of (h.min_height)), tostring (word_of (floor_value)))) end
	if args.original ~= nil then
		-- the same setup in the ORIGINAL-solver profile (the clearance_original report): its first hand-off must be this one, field for field
		local file = io.open (args.original, "rb")
		if file == nil then return verdict ("FAILURE", "cannot read original=" .. args.original) end
		local original
		for line in file:lines () do
			if string.find (line, "^sh%-witness%-handoff ordinal=1 ") then original = line end
		end
		file:close ()
		if original == nil then return verdict ("FAILURE", "the original report holds no first hand-off") end
		local mine = { host_id = h.host_id, elevation = bits_or (h.elevation), min_height = bits_or (h.min_height), pre_y = bits_or (h.pre_y), answered_y = bits_or (h.answered_y), phase = tostring (h.phase) }
		local differing = {}
		for _, key in ipairs ({ "host_id", "elevation", "min_height", "pre_y", "answered_y", "phase" }) do
			local theirs = string.match (original, " " .. key .. "=(%S+)")
			if theirs ~= mine [key] then differing [#differing + 1] = key .. " (original " .. tostring (theirs) .. ", hosted " .. tostring (mine [key]) .. ")" end
		end
		if #differing > 0 then return verdict ("FAILURE", "the hosted first hand-off differs from the original profile's: " .. table.concat (differing, "; ")) end
		state.original_equal = true
	end
	return verdict ("PASS", string.format ("%s: window %d, consumed word %s, elevation %s, hand-off %s >= max(elevation, word - elevation)", mode, h.window, tostring (h.real_word), tostring (word_of (h.elevation)), tostring (word_of (h.min_height))))
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

local function save_once (name)
	state.context = "save"
	state.pack_reads, state.pack_order, state.pack_duplicates, state.other_during_save = {}, {}, 0, 0
	local ok, text = guarded ("save", function () return engine:sector_height_witness_save (name) end)
	state.context = "frames"
	if ok then state.save_line = first_line (text) end
	return ok
end

local function read_expect (path)
	local file = io.open (path, "rb")
	if file == nil then refuse ("cannot read expect=" .. path) end
	local packed, order = {}, {}
	for line in file:lines () do
		local x, z, word = string.match (line, "^sh%-save%-word i=%d+ x=(%d+) z=(%d+) word=(0x%x%x%x%x%x%x%x%x)%s*$")
		if x ~= nil then
			local key_xz = x .. "," .. z
			if packed [key_xz] ~= nil then refuse ("the save report holds the sector " .. key_xz .. " twice") end
			packed [key_xz] = string.lower (word)
			order [#order + 1] = key_xz
		end
	end
	file:close ()
	return packed, order
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

local function session_verdict ()
	if #state.reply_faults > 0 then return verdict ("FAILURE", "the host's own answer broke the contract: " .. state.reply_faults [1]) end
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
	if #state.pack_order < 1 then return verdict ("FAILURE", "the save packed no sector word") end
	if state.pack_duplicates > 0 then return verdict ("FAILURE", "the pack read a sector twice (" .. state.pack_duplicates .. ")") end
	if mode == "save" then
		for _, key_xz in ipairs (seed_order) do
			local row = state.pack_reads [key_xz]
			if row == nil then return verdict ("FAILURE", "the seeded sector " .. key_xz .. " was never packed: not a sector of this map's session") end
			if row.word ~= seeds [key_xz].word then return verdict ("FAILURE", "the seeded sector " .. key_xz .. " was packed as " .. row.word .. ", not " .. seeds [key_xz].word) end
		end
		return verdict ("PASS", string.format ("save: %d sectors packed, %d seeded sectors packed with their seeds' bits, %s", #state.pack_order, #seed_order, state.save_line))
	end
	-- the load
	local packed, packed_order = read_expect (args.expect)
	if #state.restore_order < 1 then return verdict ("UNREACHED", "load: the boot made no restore: the saved script was not loaded as a saved campaign") end
	if state.restore_duplicates > 0 then return verdict ("FAILURE", "a sector was restored more than once (" .. state.restore_duplicates .. ")") end
	if #state.restore_order ~= #packed_order then return verdict ("FAILURE", string.format ("the load restored %d sectors, the save packed %d", #state.restore_order, #packed_order)) end
	for _, key_xz in ipairs (packed_order) do
		local row = state.restores [key_xz]
		if row == nil then return verdict ("FAILURE", "the sector " .. key_xz .. " was packed but never restored") end
		if row.stated ~= packed [key_xz] then return verdict ("FAILURE", string.format ("the sector %s was restored as %s, the save packed %s", key_xz, row.stated, packed [key_xz])) end
		if row.stored ~= row.stated then return verdict ("FAILURE", "the host did not acknowledge the bits stated for " .. key_xz) end
	end
	if state.inserts_before_first_restore > 0 then return verdict ("FAILURE", string.format ("%d entity insertions came before the first restore: restore-before-insert is not shown", state.inserts_before_first_restore)) end
	if state.inserts_between_restores > 0 then return verdict ("FAILURE", string.format ("%d entity insertions came between two restores: restore-before-insert is not shown", state.inserts_between_restores)) end
	if state.inserts_since_restore < 1 then return verdict ("UNREACHED", "load: no entity insertion followed the restores: restore-before-insert and no-lowering are not exercised") end
	for _, key_xz in ipairs (packed_order) do
		local final = state.pack_reads [key_xz]
		if final == nil then return verdict ("FAILURE", "the second pack never read the sector " .. key_xz) end
		local rel = relation (state.restores [key_xz].stated, final.word)
		if rel == "violation" then return verdict ("FAILURE", string.format ("the sector %s was restored as %s and read back as %s: LOWERED or a NaN changed", key_xz, state.restores [key_xz].stated, final.word)) end
	end
	if #state.pack_order ~= #packed_order then return verdict ("FAILURE", string.format ("the second pack read %d sectors, the first %d", #state.pack_order, #packed_order)) end
	return verdict ("PASS", string.format ("load: %d sectors restored exactly as saved and acknowledged, all before the first entity insertion (%d after the last restore), none lowered", #state.restore_order, state.inserts_since_restore))
end

local function report ()
	emit (string.format ("sh-witness-identity schema=1 mode=%s scenario=%s seed=%s frame_ms=%s hours=%s window=%s", mode, scenario_name, tostring (seed), tostring (frame_ms), tostring (hours), tostring (target_window)))
	if plan.family == "clearance" then
		emit (string.format ("sh-witness-counts windows=%d windows_without_handoff=%d scan_reads=%d height_reads=%d altered_windows=%d altered_reads=%d handoffs=%d frames=%s", state.window, state.windows_without_handoff, state.scan_reads, state.height_reads, state.altered_windows, state.altered_reads, state.handoff_count, tostring (state.frames_run)))
		local h = state.handoff
		if h ~= nil then
			emit (string.format ("sh-witness-handoff ordinal=%d window=%d reads=%d sector=%s real_word=%s answered_word=%s host_id=%s elevation=%s min_height=%s pre_y=%s answered_y=%s phase=%s", h.ordinal, h.window, h.reads, h.sector, h.real_word, h.answered_word, h.host_id, tostring (bits_or (h.elevation)), tostring (bits_or (h.min_height)), tostring (bits_or (h.pre_y)), tostring (bits_or (h.answered_y)), tostring (h.phase)))
			if state.original_equal then emit ("sh-witness-original equal=1 fields=host_id,elevation,min_height,pre_y,answered_y,phase") end
			if plan.intervene == "word" then
				local physical = "not_compared"
				if args.control_min ~= nil and args.control_answer_y ~= nil then
					physical = (word_of (h.answered_y) ~= string.sub (string.lower (args.control_answer_y), 3)) and "changed" or "unchanged"
					emit (string.format ("sh-witness-effect handoff=%s physical=%s", (word_of (h.min_height) ~= string.sub (string.lower (args.control_min), 3)) and "changed" or "unchanged", physical))
				else
					emit ("sh-witness-effect handoff=not_compared physical=not_compared")
				end
			end
		end
	else
		emit (string.format ("sh-witness-counts height_reads=%d pack_reads=%d pack_other_questions=%d restores=%d restore_duplicates=%d inserts=%d inserts_before_first_restore=%d inserts_between_restores=%d inserts_after_last_restore=%d seeds_checked=%d", state.height_reads, #state.pack_order, state.other_during_save, #state.restore_order, state.restore_duplicates, state.inserts, state.inserts_before_first_restore, state.inserts_between_restores, state.inserts_since_restore, state.seeded_checked))
		if state.save_line then emit (state.save_line) end
		for _, key_xz in ipairs (seed_order) do emit (string.format ("sh-save-seed sx=%s sz=%s word=%s", seeds [key_xz].x, seeds [key_xz].z, seeds [key_xz].word)) end
		if mode == "save" then
			for index, key_xz in ipairs (state.pack_order) do
				local row = state.pack_reads [key_xz]
				emit (string.format ("sh-save-word i=%d x=%d z=%d word=%s", index - 1, row.x, row.z, row.word))
			end
		end
		if loading then
			for index, key_xz in ipairs (state.restore_order) do
				local row = state.restores [key_xz]
				local final = state.pack_reads [key_xz]
				emit (string.format ("sh-load-restore i=%d x=%d z=%d stated=%s stored=%s final=%s relation=%s", index - 1, row.x, row.z, row.stated, row.stored, final and final.word or "none", final and relation (row.stated, final.word) or "none"))
			end
		end
	end
	if fatal ~= nil then emit (string.format ("sh-witness-fatal where=%s marker=%s line=%s", fatal.where, tostring (fatal.marker), string.gsub (fatal.line, "%s+", " "))) end
	for _, fault in ipairs (state.reply_faults) do emit ("sh-witness-reply-fault " .. string.gsub (fault, "%s+", " ")) end
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
