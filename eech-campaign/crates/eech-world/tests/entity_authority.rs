//! The host-authority and refusal witness of the production ENTITY_COLLISION narrow-phase handoff (EEDCS-49, D3): DRIVER CONTROLS.
//!
//! `lua/entity-authority.lua` boots the campaign through the public path (`require ("eech_dc")`, a DLL built with `weapon-fixture`), launches ONE real weapon through the campaign's own launch
//! call and observes what the host's MODEL provider is asked (`entity_collision`) while the weapon flies. The public witness needs a retail installation and the DLL, so it is Root's native run.
//! This file runs the REAL driver, in a real Lua state, against SCRIPTED collaborators (a scripted `eech_dc` engine, a scripted `host.model`, a scripted exclusive output file), to show what the
//! driver does with: a reached case and one that asked the host nothing, every UNREACHED distinction (no entity lookup, no precision, no candidate), a malformed request or reply, a forced hit
//! that the caller did and did not use, a forced miss, each refusal at the anchored engine stop with the exact marker, an incomplete or inconsistent cohort, the Original profile's red, exact
//! delegation of every other operation and the physical signature of the cohort. The collaborators invent no physics and say nothing about the DLL or the host's kernel; the scripted engine
//! applies only as much of the real bridge and C domain rules as the driver's bookkeeping needs (the rules themselves are checked in eech-engine-sys `entity_dialogue` and eech-dc).

use mlua::Lua;
use serde_json::Value;

const SCRIPT: &str = include_str!("../lua/entity-authority.lua");
const TREE_SCRIPT: &str = include_str!("../lua/tree-authority.lua");

const COLLABORATORS: &str = r#"
WRITTEN, CLOSED, CREATED = {}, 0, false
AIM_LOG, CALLER_MUTATED, ORIGINAL = {}, false, nil
MODEL_LOG, DELEGATION_BROKEN, ENTITY_LOG, LAUNCH = {}, {}, {}, nil
CAND_LOG, CAND_POS = {}, 0
SCAN_LOG = {}
-- every roster event (the sector lists' insertions, removals and boots; the same question, phases 4 to 6) the REAL provider answered: phase pattern, result, the key it named and the identity place the question carried
ROSTER_LOG = {}
-- every city event (the buildings of a city: phases 7 to 12) the REAL provider answered: phase pattern, result, the key it named and the identity place the question carried
CITY_LOG = {}
-- every FIXED_STATE report (phase 13: a fixed placement's collision state) the REAL provider answered: phase pattern, result, the key it named and the identity place the question carried
FIXED_LOG = {}
-- every CITY_EXPAND question (phase 15, Task49) the scripted provider answered: phase pattern, result, the city key, the identity place
EXPAND_LOG, EXPANDED = {}, {}
POSE_LOG = {}
MOVES, RETIRED, DISPLACED, MOVE_LOG = {}, {}, {}, {}
VECTORS_ASKED, VECTORS_ALTERED, VECTORS_SEEN = 0, false, nil
MQ_CALLS = {}
-- every move step the host received: id|demand received|phase|fields that differ from the engine's own step ('-' is the same table)|demand the engine gave; and the last demand the host received per identity
DEMAND_LOG, LAST_DEMAND, CURRENT_STEP = {}, {}, nil
-- a roster identity of 128 bytes mixing space, newline, tab, carriage return, `%`, `-`, `=` and UTF-8 (the maximum of the domain)
EXOTIC_ID = (function ()
	local cycle = { 32, 10, 97, 9, 37, 195, 169, 45, 61, 13, 90, 226, 130, 172, 126, 46, 95, 57 }
	local bytes = {}
	for k = 1, 128 do bytes [k] = string.char (cycle [(k - 1) % #cycle + 1]) end
	return table.concat (bytes)
end) ()
-- the C encoder of csrc/eech_mq_report.h: every byte but an ASCII letter, digit or `._~` is %XX in upper-case hexadecimal
local function mq_enc (id) return (string.gsub (id, "[^A-Za-z0-9._~]", function (c) return string.format ("%%%02X", string.byte (c)) end)) end
local MQ_BEARING_WORDS = { "0x3f060a92", "0x40060a92", "0x406a927f" }
-- one single fault applied to a REAL-shaped report (the scenario's `query_fault`)
local function mq_fault (name, text)
	local lines = {}
	for line in string.gmatch (text, "[^\n]+") do lines [#lines + 1] = line end
	local function join () return table.concat (lines, "\n") .. "\n" end
	if name == "prepare_unknown_record" then lines [#lines + 1] = "wpn-moving-query-noise x=1"
	elseif name == "prepare_missing_segment" then lines [#lines] = nil
	elseif name == "prepare_duplicate_segment" then lines [4] = lines [3]
	elseif name == "prepare_other_target" then lines [1] = string.gsub (lines [1], "host_id=[^ ]+", "host_id=eech%%2Dworld%%3A3")
	elseif name == "prepare_bad_bearing" then lines [2] = string.gsub (lines [2], "bearing=0x%x+", "bearing=0x3f800000")
	elseif name == "prepare_nonfinite_endpoint" then lines [2] = string.gsub (lines [2], " a=0x%x+", " a=0x7f800000", 1)
	elseif name == "prepare_wrong_type" then lines [1] = string.gsub (lines [1], "ENTITY_TYPE_ROUTED_VEHICLE", "routed_vehicle")
	elseif name == "prepare_double_space" then lines [2] = string.gsub (lines [2], " bearing", "  bearing")
	elseif name == "prepare_extra_key" then lines [1] = lines [1] .. " extra=1"
	elseif name == "prepare_no_newline" then return (string.gsub (join (), "\n$", ""))
	elseif name == "run_extra_row" then lines [#lines + 1] = lines [#lines]
	elseif name == "run_missing_row" then lines [#lines] = nil
	elseif name == "run_other_target" then lines [1] = string.gsub (lines [1], "host_id=[^ ]+", "host_id=eech%%2Dworld%%3A3")
	elseif name == "run_hit_without_point" then lines [2] = string.gsub (lines [2], " point=0x%x+,0x%x+,0x%x+", " point=-")
	elseif name == "run_hit_two_normal_words" then lines [2] = string.gsub (lines [2], "normal=(0x%x+,0x%x+),0x%x+", "normal=%1")
	elseif name == "run_miss_with_point" then lines [3] = string.gsub (lines [3], "point=%-", "point=0x42000000,0x42000001,0x42000002")
	elseif name == "run_miss_with_identity" then lines [3] = string.gsub (lines [3], "hit_host_id=%-", "hit_host_id=eech%%2Dworld%%3A2")
	elseif name == "run_target_flag_with_other_identity" then lines [2] = string.gsub (lines [2], "hit_host_id=[^ ]+", "hit_host_id=eech%%2Dworld%%3A3")
	elseif name == "run_target_identity_with_false_flag" then lines [2] = string.gsub (lines [2], "hit_is_target=1", "hit_is_target=0")
	elseif name == "run_target_identity_with_false_flag_and_other_type" then
		lines [2] = string.gsub (string.gsub (lines [2], "hit_is_target=1", "hit_is_target=0"), "hit_type=[A-Z_]+", "hit_type=ENTITY_TYPE_HELICOPTER")
	elseif name == "run_bad_encoding" then lines [2] = string.gsub (lines [2], "hit_host_id=[^ ]+", "hit_host_id=a%%zz")
	elseif name == "run_lowercase_escape" then lines [2] = string.gsub (lines [2], "hit_host_id=[^ ]+", "hit_host_id=a%%c3%%a9")
	elseif name == "run_non_canonical_escape" then lines [2] = string.gsub (lines [2], "hit_host_id=[^ ]+", "hit_host_id=%%41")
	elseif name == "run_short_word" then lines [2] = string.gsub (lines [2], "point=0x42000000", "point=0x4200")
	elseif name == "run_unknown_key" then lines [2] = string.gsub (lines [2], " hit_is_target=", " ghost=1 hit_is_target=")
	elseif name == "run_reordered_keys" then lines [2] = string.gsub (lines [2], "(hit=%d) (a=[^ ]+)", "%2 %1")
	elseif name == "run_bad_hit_flag" then lines [2] = string.gsub (lines [2], " hit=1", " hit=2")
	elseif name == "run_hit_is_target_unknown" then lines [2] = string.gsub (lines [2], "hit_is_target=1", "hit_is_target=2")
	elseif name == "run_trailing_space" then lines [2] = lines [2] .. " "
	else error ("unknown scripted fault " .. tostring (name)) end
	return join ()
end
local function deep_copy (value)
	if type (value) ~= "table" then return value end
	local copy = {}
	for key, inner in pairs (value) do copy [key] = deep_copy (inner) end
	return copy
end
-- the paths at which two requests differ; numbers are compared by their printed form so that a signed zero is a difference
local function differences (a, b, path, out)
	if type (a) == "table" and type (b) == "table" then
		local seen = {}
		for key in pairs (a) do seen [key] = true end
		for key in pairs (b) do seen [key] = true end
		for key in pairs (seen) do
			local inner = type (key) == "number" and (path .. "[" .. key .. "]") or (path == "" and key or (path .. "." .. key))
			differences (a [key], b [key], inner, out)
		end
	elseif type (a) == "number" and type (b) == "number" then
		if string.format ("%g", a) ~= string.format ("%g", b) then out [#out + 1] = path end
	elseif a ~= b then
		out [#out + 1] = path
	end
end
local scenario = SCENARIO
local file = { writes = 0, flushes = 0 }
function file:write (text)
	self.writes = self.writes + 1
	local fail = scenario.io_fail
	if fail and fail.op == "write" and fail.at == self.writes then return nil, "disk full" end
	WRITTEN [#WRITTEN + 1] = text
	return self
end
function file:flush ()
	self.flushes = self.flushes + 1
	local fail = scenario.io_fail
	if fail and fail.op == "flush" and fail.at == self.flushes then return nil, "flush failed" end
	return true
end
function file:close ()
	CLOSED = CLOSED + 1
	local fail = scenario.io_fail
	if fail and fail.op == "close" then return nil, "close failed" end
	return true
end

-- the words of the scripted segment: OLD (100, 50, 200) and NEW (300, 60, 400)
local OLD_W = { "0x42c80000", "0x42480000", "0x43480000" }
local NEW_W = { "0x43960000", "0x42700000", "0x43c80000" }
local ZERO = "0x00000000"
local UP = ZERO .. ",0x3f800000," .. ZERO
local NO_POINT = ZERO .. "," .. ZERO .. "," .. ZERO
-- the request as the corrected contract sends it: the segment's two ends, then the candidate's pose (12 words stationary, 16 moving); never the caller's outputs
local function pose_key (moving)
	local words = { OLD_W [1], OLD_W [2], OLD_W [3], NEW_W [1], NEW_W [2], NEW_W [3] }
	for _ = 1, moving and 16 or 12 do words [#words + 1] = "0x3f800000" end
	if scenario.bad_request == "count" then words [#words] = nil
	elseif scenario.bad_request == "token" then words [7] = "1.5"
	elseif scenario.bad_request == "extra" then return table.concat (words, ","), 1 end
	return table.concat (words, ",")
end

-- the REAL model provider as the scripted host.model: every call is logged with its arguments and the table it answered; the entity answer is the scenario's `real`
local call_no, ANSWERS = 0, {}
-- the scripted host's roster: how many keys it issued, the key of the one entry the scripted sector holds, and the last key that ended
local ROSTER = { issued = 0, live = nil, ended = nil }
local function reference (op, scene, depth, kind, definition, key)
	call_no = call_no + 1
	local record = { op = op, scene = scene, depth = depth, kind = kind, definition = definition, key = key }
	MODEL_LOG [#MODEL_LOG + 1] = record
	local answer
	if op == "entity_collision" then
		if scenario.real_error then error ("the real provider refuses", 0) end
		local first3 = string.match (key, "^([^,]+,[^,]+,[^,]+),") or NO_POINT
		local real = scenario.real or "miss"
		if real == "hit" then answer = { result = "hit", mask = 63, name = "CANDIDATE", p = first3 .. "," .. UP }
		elseif real == "unassigned_word" then answer = { result = "miss", mask = 56, name = "CANDIDATE", p = "0x00000001," .. ZERO .. "," .. ZERO .. "," .. UP }
		elseif real == "hit_mask" then answer = { result = "hit", mask = 56, name = "CANDIDATE", p = NO_POINT .. "," .. UP }
		elseif real == "partial" then answer = { result = "miss", mask = 1, name = "CANDIDATE", p = "0x3f800000," .. ZERO .. "," .. ZERO .. "," .. NO_POINT }
		else answer = { result = "miss", mask = 56, name = "CANDIDATE", p = NO_POINT .. "," .. UP } end
	elseif op == "entity_scan" then
		-- the REAL answers (mb_hostentity.h grammar), by the phase the campaign reports. The question is the 16 facts and, when it carries one, a space and the identity place: the key of the entry the scan
		-- stands at (ENTRY and RESOLVED), a hosted mobile's identity (an INSERT) or the key a REMOVE names. The host keeps ONE entry in the scripted sector (0, 0): INSERT issues its key (never reused), the
		-- start and the end of a scan serve it, an offer is made for it when it is not excluded, a miss goes to the end of the list and a hit ends the scan
		if scenario.real_scan_error then error ("the real provider refuses", 0) end
		local words = {}
		for token in string.gmatch (string.sub (key, 1, 175), "[^,]+") do words [#words + 1] = token end
		local identity = string.sub (key, 177)
		local phase, excluded, hit = words [12], words [15] ~= ZERO, words [16] ~= ZERO
		local bad, bad_roster = scenario.bad_real_scan, scenario.bad_real_roster
		local CITY_PHASES = { ["0x40e00000"] = true, ["0x41000000"] = true, ["0x41100000"] = true, ["0x41200000"] = true, ["0x41300000"] = true, ["0x41400000"] = true }
		if CITY_PHASES [phase] and not scenario.real_scan then error ("a city event needs the real provider (real_scan)", 0) end
		local FIXED_PHASE, CITY_STATE_PHASE = "0x41500000", "0x41600000"
		if phase == FIXED_PHASE and not scenario.real_scan then error ("a fixed state report needs the real provider (real_scan)", 0) end
		if phase == CITY_STATE_PHASE and not scenario.real_scan then error ("a city state report needs the real provider (real_scan)", 0) end
		if phase == "0x41700000" then
			if not scenario.real_scan then error ("a city expansion needs the real provider (real_scan)", 0) end
			local city, child = string.match (identity, "^(%d+) (%d+)$")
			if child ~= nil and ROSTER_HOLD (child, true) == "withdrawn" then
				answer = { result = "refused", reason = "[roster.city_box.building_state] the host withdrew the collision state of the city child " .. child }
			else
				answer = { result = "expanded", name = city or "", p = scenario.expanded_p or "0xc0800000,0xc1200000,0xc0000000,0x40000000,0x40000000,0x40c00000" }
			end
			if scenario.bad_real_expand == "state" then answer.result = "placed" end
			if scenario.bad_real_expand == "echo" then answer.name = "99" end
			if scenario.bad_real_expand == "short" then answer.p = "0xc0800000,0xc1200000" end
			if scenario.bad_real_expand == "infinite" then answer.p = "0xc0800000,0xc1200000,0xc0000000,0x40000000,0x40000000,0x7f800000" end
			-- the same 65 bytes as a valid box, but not six exact patterns: five patterns with fifteen separators, a signed pseudo-hex word and a blank one (each passes a length-and-token-count check)
			if scenario.bad_real_expand == "commas" then answer.p = "0xc0800000,,,0xc1200000,,,0xc0000000,,,0x40000000,,,0x40000000,,," end
			if scenario.bad_real_expand == "signed" then answer.p = "0xc0800000,0x+1234567,0xc0000000,0x40000000,0x40000000,0x40c00000" end
			if scenario.bad_real_expand == "blank" then answer.p = "0xc0800000,0x 1234567,0xc0000000,0x40000000,0x40000000,0x40c00000" end
			EXPAND_LOG [#EXPAND_LOG + 1] = { phase = phase, result = answer.result, name = answer.name or "", identity = identity }
		elseif scenario.real_scan then
			-- the REAL provider (eech-world's model_host with its own SectorRoster, registered by the test as REAL_HOST): every question of the scan and the roster is answered by it
			-- the host's restatement of a record's sub type (UNKNOWN, no withdrawal word) the provider FAILS to answer at all (the real provider never does)
			if phase == FIXED_PHASE and scenario.bad_real_fixed == "restatement_error" and words [10] == ZERO and words [11] == ZERO then error ("the real provider refuses the restatement", 0) end
			answer = REAL_HOST.model (op, scene, depth, kind, definition, key)
		elseif phase == "0x40800000" then
			ROSTER.issued = ROSTER.issued + 1
			ROSTER.live = tostring (ROSTER.issued)
			answer = { result = "placed", name = ROSTER.live }
			if bad_roster == "reused_key" and ROSTER.issued > 1 then answer.name = tostring (ROSTER.issued - 1) end
			if bad_roster == "bad_key" then answer.name = "01" end
			if bad_roster == "no_key" then answer.name = "" end
			if bad_roster == "state" then answer.result = "removed" end
		elseif phase == "0x40a00000" then
			ROSTER.ended, ROSTER.live = ROSTER.live, nil
			-- a removal may name the CITY record the host keeps for the box it built (a roster key); a name that is no key is a reply fault
			answer = { result = "removed", name = (bad_roster == "key_on_remove" and "x7") or (bad_roster == "record_on_remove" and "7") or "" }
		elseif phase == "0x40c00000" then
			ROSTER.live = nil
			answer = { result = "cleared", name = "" }
		elseif phase == ZERO then
			answer = scenario.scan_offers_nothing and { result = "done_none" } or { result = "first", p = ZERO .. "," .. ZERO, name = ROSTER.live or "" }
			if bad == "ended_key_served" and ROSTER.ended then answer.name = ROSTER.ended end
		elseif phase == "0x3f800000" then answer = { result = excluded and "next" or "offer", name = "" }
		elseif phase == "0x40000000" then answer = { result = "done_none", name = "" }
		else answer = { result = hit and "done_hit" or "next", name = "" } end
		if bad == "state" then answer.result = "perhaps" end
		if bad == "p_on_next" and answer.result ~= "first" then answer.p = ZERO .. "," .. ZERO end
		if bad == "name" then answer.name = "SECTOR" end
		if bad == "name_on_offer" and answer.result == "offer" then answer.name = "5" end
		if bad == "offer_for_resolved" and phase == "0x40400000" then answer.result = "offer" end
		if bad == "first_one_pattern" and answer.result == "first" then answer.p = ZERO end
		if bad_roster == "p_on_event" and (phase == "0x40800000" or phase == "0x40a00000") then answer.p = ZERO .. "," .. ZERO end
		-- a reply fault of the city events, forced AFTER the real answer (the real provider never gives one)
		local bad_city = scenario.bad_real_city
		if CITY_PHASES [phase] and bad_city then
			if bad_city == "state" and phase == "0x41100000" then answer.result = "placed" end
			if bad_city == "insert_no_key" and phase == "0x40e00000" then answer.name = "" end
			if bad_city == "removed_key" and phase == "0x41000000" then answer.name = "9" end
			if bad_city == "link_echo" and phase == "0x41300000" then answer.name = "99" end
			if bad_city == "name_echo" and phase == "0x41400000" then answer.name = "99" end
			if bad_city == "served_not_key" and (phase == "0x41100000" or phase == "0x41200000") and answer.name ~= "" then answer.name = "01" end
			if bad_city == "reused_key" and phase == "0x40e00000" and answer.name ~= "2" then answer.name = "1" end
		end
		-- a reply fault of the fixed state report, forced AFTER the real answer (the real provider never gives one)
		if (phase == FIXED_PHASE or phase == CITY_STATE_PHASE) and scenario.bad_real_fixed == "echo" then answer.name = "99" end
		if (phase == FIXED_PHASE or phase == CITY_STATE_PHASE) and scenario.bad_real_fixed == "state" then answer.result = "removed" end
		-- ONLY the host's restatement of a record's sub type (the sub type word UNKNOWN, no withdrawal word) is damaged: a cursor beside a correct `placed` and the same key. The campaign's own report keeps its real answer
		if phase == FIXED_PHASE and scenario.bad_real_fixed == "restatement_cursor" and words [10] == ZERO and words [11] == ZERO then answer.p = ZERO .. "," .. ZERO end
		-- the same restatement acknowledged under ANOTHER placement's key, or answered with another state (each keeps the real state change the provider made)
		if phase == FIXED_PHASE and scenario.bad_real_fixed == "restatement_echo" and words [10] == ZERO and words [11] == ZERO then answer.name = "99" end
		if phase == FIXED_PHASE and scenario.bad_real_fixed == "restatement_state" and words [10] == ZERO and words [11] == ZERO then answer.result = "removed" end
		if phase == "0x41700000" then
			-- logged above
		elseif CITY_PHASES [phase] then
			CITY_LOG [#CITY_LOG + 1] = { phase = phase, result = answer.result, name = answer.name or "", identity = identity }
		elseif phase == FIXED_PHASE or phase == CITY_STATE_PHASE then
			FIXED_LOG [#FIXED_LOG + 1] = { phase = phase, result = answer.result, name = answer.name or "", identity = identity }
		elseif phase == "0x40800000" or phase == "0x40a00000" or phase == "0x40c00000" then
			ROSTER_LOG [#ROSTER_LOG + 1] = { phase = phase, result = answer.result, name = answer.name, identity = identity }
		else
			SCAN_LOG [#SCAN_LOG + 1] = { phase = phase, result = answer.result, key = key, standing = identity }
		end
	elseif op == "entity_pose" then
		-- the REAL observation of a mobile candidate: `recorded` (a routed vehicle's record, no motion vector held) for a named identity and `no_id` for an unnamed one, unless the scenario says otherwise
		if scenario.real_pose_error then error ("the real provider refuses", 0) end
		local named = #key > 164
		local result = named and (scenario.pose_class or "recorded") or "no_id"
		local recorded = result == "recorded"
		answer = { result = result, kind = recorded and (scenario.pose_kind or 1) or 0, difference = recorded and ((depth == 1 and scenario.pose_difference_placement) or scenario.pose_difference or 8) or 0 }
		if DISPLACED [string.sub (key, 166)] then answer = { result = "recorded", kind = 1, difference = 1 } end
		local bad = scenario.bad_real_pose
		if bad == "state" then answer.result = "perhaps" end
		if bad == "kind" then answer.kind = 6 end
		if bad == "no_id_for_named" then answer = { result = "no_id", kind = 0, difference = 0 } end
		if bad == "attitude_at_offer" then answer = { result = "recorded", kind = 1, difference = 2 } end
		if bad == "contradiction" then answer = { result = "recorded", kind = 1, difference = 12 } end
		if bad == "kind_for_unrecorded" then answer = { result = "unknown", kind = 1, difference = 0 } end
		POSE_LOG [#POSE_LOG + 1] = { stage = depth, entity_type = kind, alive = definition, result = answer.result, key = key }
	elseif op == "entity_candidate" then
		-- the REAL candidate answers of one dialogue, in the scenario's order (a scan that offers one mobile candidate and is told to test it, by default); the name is the scene's
		CAND_POS = CAND_POS + 1
		local state = (scenario.candidate_sequence or { "test" }) [CAND_POS] or "test"
		if scenario.real_candidate_error then error ("the real provider refuses", 0) end
		local identity = #key > 483 and string.sub (key, 485) or ""
		if RETIRED [identity] then
			answer = { result = "refused", reason = "candidate record: the host holds no physical record for this mobile candidate (the host retired the identity)" }
			ANSWERS [#ANSWERS + 1] = answer
			record.answer = answer
			return answer
		end
		-- a non-mobile candidate of a record whose fixed state the host withdrew (the real roster says so; this reply only states it in the provider's words, which tests/model_target_point.rs pins against the real provider)
		-- the same for a CITY whose own position and box the host withdrew, at the stage that reads them only: its fixed facts read (43), fixed (8), a city (9), a first building served (12), not under a city (11)
		local cw = {}
		for token in string.gmatch (string.sub (key, 1, 483), "[^,]+") do cw [#cw + 1] = token end
		if scenario.city_state and identity ~= "" and cw [43] ~= ZERO and cw [8] ~= ZERO and cw [9] ~= ZERO and cw [12] ~= ZERO and cw [11] == ZERO and ROSTER_HOLD (identity) == "withdrawn" then
			answer = { result = "refused", reason = "[candidate.city] the host withdrew the collision state of this city" }
			ANSWERS [#ANSWERS + 1] = answer
			record.answer = answer
			return answer
		end
		-- a record whose derived rule is no collision (the host restated its sub type) against a campaign whose collision-fixed flag (8) is true at the fixed facts' stage (43): the provider's own named mirror refusal
		-- (the real join compares the campaign's flag once it was GIVEN: an ordinary fixed candidate at its fixed facts (43), a building listed under a city (11) only at its placement facts (44))
		if (scenario.fixed_state or scenario.building_state) and identity ~= "" and ((cw [11] ~= ZERO and cw [44] ~= ZERO) or (cw [11] == ZERO and cw [43] ~= ZERO)) and cw [8] ~= ZERO and ROSTER_HOLD (identity, cw [11] ~= ZERO) == "no_collision" then
			answer = { result = "refused", reason = "[candidate.mirror] the campaign's collision-fixed flag differs from the host's fixed state of the placement" }
			ANSWERS [#ANSWERS + 1] = answer
			record.answer = answer
			return answer
		end
		if scenario.fixed_state and identity ~= "" and ROSTER_HOLD (identity, cw [11] ~= ZERO) == "withdrawn" then
			answer = { result = "refused", reason = "[candidate.fixed] the host withdrew the fixed physical state of this placement" }
			ANSWERS [#ANSWERS + 1] = answer
			record.answer = answer
			return answer
		end
		local bad = scenario.bad_real_candidate
		answer = { result = bad == "state" and "perhaps" or state, name = (scene == 0 and bad ~= "name0") and "" or "CANDIDATE" }
		if bad == "name0" then answer.name = "SOMETHING" end
		if bad == "unnamed" then answer.name = "" end
		if state == "test" or bad == "pose" then
			local count = (bad == "pose") and 13 or (scenario.moving and 16 or 12)
			local words = {}
			for _ = 1, count do words [#words + 1] = "0x3f800000" end
			answer.p = table.concat (words, ",")
		end
		-- ONE fault of the FINAL positive only (every earlier answer stays coherent): a non-finite pose word or another scene's name; `subnormal` (a subnormal word and a negative zero) is a VALID positive
		local fault = scenario.final_fault
		if state == "test" and fault then
			local pose = {}
			for token in string.gmatch (answer.p, "[^,]+") do pose [#pose + 1] = token end
			if fault == "inf" then pose [3] = "0x7f800000"
			elseif fault == "ninf" then pose [3] = "0xff800000"
			elseif fault == "snan" then pose [3] = "0x7fa00001"
			elseif fault == "subnormal" then pose [3], pose [4] = "0x00000001", "0x80000000"
			elseif fault == "name" and answer.name ~= "" then answer.name = "OTHER" end
			answer.p = table.concat (pose, ",")
		end
		CAND_LOG [#CAND_LOG + 1] = { scene = scene, result = answer.result, key = key }
	else
		answer = { op = op, token = call_no }
	end
	ANSWERS [#ANSWERS + 1] = answer
	record.answer = answer
	return answer
end

local ids = 0
local profile_stage
host = {
	args = ARGS,
	use_reference_arithmetic = function ()
		assert (profile_stage == "prepared", "the profile must be selected right after the installation is prepared")
		profile_stage = "selected"
	end,
	bind_mobile = function () ids = ids + 1 return "eech-world:" .. ids end,
	-- the unmodified host aim: it records every request it receives, the paths where it differs from the engine's pristine request, and whether a signed zero survived
	aim = function (request)
		if request.mode == "vectors" then VECTORS_SEEN = request end
		local paths = {}
		differences (ORIGINAL, request, "", paths)
		table.sort (paths)
		local answers = request.answers
		local zero = type (answers) == "table" and type (answers [1]) == "table" and type (answers [1].f) == "table" and string.format ("%g", answers [1].f [1]) or "-"
		AIM_LOG [#AIM_LOG + 1] = { mode = request.mode, launched = ARMED_NOW, paths = table.concat (paths, ","), zero = zero, request = request }
		return { state = "done" }
	end,
	env_provider = function () return function () return {} end end,
	terrain_provider = function () return function () return { elevation = 0 } end end,
	model = reference,
	move_vehicle = function (step)
		MOVES [#MOVES + 1] = step
		MOVE_LOG [#MOVE_LOG + 1] = { id = step.host_id, mode = step.mode, elevation = step.terrain and step.terrain.elevation or -1, launched = LAUNCH ~= nil }
		if step.mode == "retire" then RETIRED [step.host_id] = true elseif step.mode == "place" then DISPLACED [step.host_id] = true end
		if step.mode == "move" then
			local differing = {}
			if step ~= CURRENT_STEP then
				for key, value in pairs (step) do if CURRENT_STEP [key] ~= value then differing [#differing + 1] = key end end
				for key in pairs (CURRENT_STEP) do if step [key] == nil then differing [#differing + 1] = key end end
				table.sort (differing)
			end
			local function shown (value) return type (value) == "number" and string.format ("%.17g", value) or tostring (value == nil and "none" or value) end
			DEMAND_LOG [#DEMAND_LOG + 1] = table.concat ({ step.host_id, shown (step.desired_velocity), #MQ_CALLS > 0 and "query" or "warmup", #differing > 0 and table.concat (differing, ",") or "-", shown (CURRENT_STEP.desired_velocity) }, "|")
			LAST_DEMAND [step.host_id] = step.desired_velocity
		end
		if step.mode == "move" then
			-- the host's physical step answers AT the terrain it was given (the convoy step sets the new y from it) unless the scenario says the answer ignores it
			local y = step.terrain and step.terrain.elevation or 0
			if scenario.answer_ignores_lift then y = scenario.ground or 10 end
			return { position = { 0, y, 0 }, velocity = 0, motion_vector = (scenario.motion or {}) [step.host_id] or { 0, 0, 0 } }
		end
		return {}
	end,
	create_exclusive = function ()
		if scenario.create_fails then return nil, "already exists" end
		CREATED = true
		return file
	end,
}
if scenario.no_model then host.model = nil end
if scenario.no_terrain then host.terrain_provider = nil end

local FATAL = "runtime error: eech: EECH fatal error: host model: "
local MODE = ARGS.mode or "control"
package.preload.eech_dc = function ()
	return {
		prepare_installation = function () profile_stage = "prepared" end,
		boot = function (config)
			assert (profile_stage == "selected", "the profile must be selected before the boot")
			local mobiles = scenario.mobiles or { { kind = 5, side = "blue" }, { kind = 1, side = "red" } }
			local positions = scenario.positions or { "0,0,0", "100,0,100" }
			local by_id, armed_weapons = {}, scenario.armed or { [1] = true }
			for index, mobile in ipairs (mobiles) do
				local id = config.bind_mobile { movement_kind = mobile.kind, side = mobile.side }
				by_id [id] = { index = index, position = positions [index] or "1000000,0,1000000" }
			end
			local last_hit = false
			local now, since_launch, launched = 0, 0, false
			local frame_no = 0
			local linked = {}
			for _, engagement in ipairs (scenario.engagements or {}) do linked ["eech-world:" .. engagement.source] = "eech-world:" .. engagement.target end
			local tests, hits, precision, flight_hit = 0, 0, 0, false
			local queue = {}
			local engine = {}
			if scenario.fixture_api then engine.collision_probe = function () end end
			function engine:clock () return { elapsed_seconds = now } end
			local function stop (marker, text) error (FATAL .. "[model-provider:" .. marker .. "] " .. text, 0) end
			-- every model operation goes through the DRIVER's wrapper; an operation the host answers itself must come back as the very table the host gave, with the arguments unchanged
			local function ask_model (op, scene, depth, kind, definition, key, expect_same)
				local before = #ANSWERS
				local ok, answer = pcall (config.model, op, scene, depth, kind, definition, key)
				if not ok then return false, answer end
				if expect_same and #ANSWERS == before + 1 then
					local record = MODEL_LOG [#MODEL_LOG]
					if answer ~= ANSWERS [#ANSWERS] or record.op ~= op or record.scene ~= scene or record.depth ~= depth or record.kind ~= kind or record.definition ~= definition or record.key ~= key then
						DELEGATION_BROKEN [#DELEGATION_BROKEN + 1] = op
					end
				elseif expect_same then
					DELEGATION_BROKEN [#DELEGATION_BROKEN + 1] = op .. " (not asked of the host exactly once)"
				end
				return true, answer
			end
			-- one weapon step as the campaign sends it: the lookup answers of the round (a first round has none), VIEW_WEAPON (kind 1) among them, with a signed zero in an unrelated word
			local function weapon_step (mode, with_answers)
				local request = { mode = mode, weapon = 7, delta_time = 0.1, position = { 1, 2, 3 }, answers = {} }
				if with_answers and not scenario.no_view_answer then
					request.answers = {
						{ kind = 3, a = 1, b = 2, c = 3, d = 4, f = { -0.0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12 } },
						{ kind = 1, a = 0, b = 0, c = 0, d = 0, f = { 0, 0, 0, 0, 0, 0, scenario.view_preset or 0, 0, 0, 0, 0, 0 } },
						{ kind = 14, a = 0, b = 0, c = 0, d = 0, f = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 } },
					}
					if scenario.short_view then request.answers [2].f [12] = nil end
				end
				ORIGINAL, ARMED_NOW = deep_copy (request), launched
				local pristine = deep_copy (request)
				config.aim (request)
				local mutated = {}
				differences (pristine, request, "", mutated)
				if #mutated > 0 then CALLER_MUTATED = true end
				local seen = AIM_LOG [#AIM_LOG].request.answers
				return with_answers and type (seen) == "table" and type (seen [2]) == "table" and seen [2].kind == 1 and seen [2].f [7] == 1 and not scenario.view_ignored
			end
			-- one scan candidate's dialogue with the host, the campaign's half as the shared C applies it (mb_hostentity.h): 44 facts staged as the host asks (a mobile candidate with its scene, or a
			-- non-mobile one with scene 0 until the fixed facts are read), the seven states, the name echo and the legality of each state against the facts. True: the narrow phase is required
			local STATES = { skip = true, need_updated = true, need_fixed = true, need_placement = true, need_normal = true, buildings = true, test = true }
			local altering = { candidate_skip = true, withheld_candidate = true, bad_candidate_state = true, bad_candidate_identity = true, withdrawn_fixed_state = true, changed_fixed_rule = true, withdrawn_city_state = true }
			-- the campaign's OBSERVATION of a mobile candidate (mb_hostentity.h ENTITY_POSE, asked beside the original's reads): the stage, the entity's type (4), alive, the 15 words and the identity of the
			-- scripted member; the reply's shape is checked as the shared C checks it, and NOTHING of the dialogue depends on the answer
			local pose_altering = { withheld_pose = true, bad_pose_answer = true, bad_pose_state = true }
			local POSE_RESULTS = { recorded = true, unrecorded = true, retired = true, unknown = true, no_id = true }
			local function pose_observe (stage, pre)
				if scenario.no_pose_questions and not pre then return end
				local words = {}
				for index = 1, 15 do words [index] = ZERO end
				words [1], words [2], words [3] = OLD_W [1], OLD_W [2], OLD_W [3]
				if stage == 1 then for index = 7, 15 do words [index] = "0x3f800000" end end
				local key = table.concat (words, ",") .. (scenario.unnamed_pose and "" or (" " .. (scenario.pose_id_length and string.rep ("x", scenario.pose_id_length) or "eech-world:1")))
				if scenario.bad_pose_request == "count" then key = string.match (key, "^(.*),[^,]+$") or key end
				local ok, answer = ask_model ("entity_pose", 0, stage, 4, 1, key, not pose_altering [MODE])
				if not ok then stop ("entity_pose.callback", tostring (answer)) end
				if type (answer) ~= "table" or not POSE_RESULTS [answer.result] then stop ("entity_pose.state", "class") end
				if type (answer.kind) ~= "number" or answer.kind < 0 or answer.kind > 5 or type (answer.difference) ~= "number" or answer.difference < 0 or answer.difference > 31 then stop ("entity_pose.answer", "range") end
				if (answer.result == "no_id") == (not scenario.unnamed_pose) then stop ("entity_pose.state", "contradicts the question") end
				if answer.result ~= "recorded" and (answer.kind ~= 0 or answer.difference ~= 0) then stop ("entity_pose.answer", "carries a record") end
				-- the stage's own masks, as the shared C checks them: no attitude is read at the offer, and a held-vector difference contradicts "none held"
				if stage == 0 and math.floor (answer.difference / 2) % 2 == 1 then stop ("entity_pose.answer", "an attitude difference at the offer stage") end
				if math.floor (answer.difference / 4) % 2 == 1 and math.floor (answer.difference / 8) % 2 == 1 then stop ("entity_pose.answer", "a held-vector difference with none held") end
				-- the CONFORMANCE assertion of the shared C: a position, attitude or motion-vector difference is a mirror the host did not make
				if answer.difference % 8 ~= 0 then stop ("entity_pose.mirror", "the campaign's words differ from the host's record") end
			end
			-- the traversal of a city's buildings (defined with the roster's events below) and the keys of the city's children, in the order they were listed
			local city_traverse
			local roster_key
			local city_children = {}
			-- the name the city's list was first given (the city's sector key at its first building): it stays the list's name when the city's placement is relinked while the buildings stay listed
			local city_list
			local function candidate_dialogue_once ()
				if scenario.no_candidate_questions then return true end
				local mobile = not scenario.fixed_candidate
				local scene = mobile and 7 or 0
				-- the key of a building listed under the city once its own offer follows the city's (a city whose own state the campaign reports)
				local child_name
				local words = {}
				for index = 1, 44 do words [index] = ZERO end
				words [1], words [2], words [3] = OLD_W [1], OLD_W [2], OLD_W [3]
				words [4], words [5], words [6] = NEW_W [1], NEW_W [2], NEW_W [3]
				if mobile then words [7] = "0x3f800000" end
				if mobile then pose_observe (0) end
				CAND_POS = 0
				for _ = 1, 8 do
					local key = table.concat (words, ",")
					if scenario.bad_candidate_request == "count" then key = string.match (key, "^(.*),[^,]+$") end
					-- a MOBILE candidate's question names its semantic host identity after the facts (the host decides its pose from its own record of it)
					if mobile and not scenario.unnamed_candidate then key = key .. " " .. (scenario.candidate_id or "eech-world:1") end
					-- every other candidate names the placement the host served it by (a roster key), on every staged question (Task49)
					if not mobile and not scenario.no_candidate_key then key = key .. " " .. (child_name or (scenario.candidate_second and scenario.second_key) or scenario.candidate_key or roster_key or "1") end
					local ok, answer = ask_model ("entity_candidate", scene, nil, nil, nil, key, not altering [MODE])
					if not ok then stop ("entity_candidate.callback", tostring (answer)) end
					if type (answer) == "table" and answer.result == "refused" then stop ("entity_candidate.refused", tostring (answer.reason)) end
					if type (answer) ~= "table" or not STATES [answer.result] then stop ("entity_candidate.answer", "result") end
					if (scene == 0 and answer.name ~= "") or (scene ~= 0 and answer.name ~= "CANDIDATE") then stop ("entity_candidate.identity", "name") end
					local result = answer.result
					if result == "skip" then return false
					elseif result == "need_updated" then
						if not mobile or words [21] ~= ZERO then stop ("entity_candidate.state", "UPDATED asked of a candidate that cannot need it") end
						words [21] = "0x40000000"
					elseif result == "need_fixed" then
						if mobile or words [43] ~= ZERO then stop ("entity_candidate.state", "fixed facts asked of a candidate that cannot need them") end
						words [43], words [8], scene = "0x3f800000", "0x3f800000", 9
						-- a building listed under the city (of_city, 11) reads its own fixed facts: the city-building flag (10), never the city's own words
						if scenario.city and words [11] ~= ZERO then words [10] = "0x3f800000"
						elseif scenario.city then words [9] = "0x3f800000" end
						if scenario.city and scenario.city_state and words [11] == ZERO then
							words [12] = "0x3f800000"
							words [14], words [15], words [16] = OLD_W [1], OLD_W [2], OLD_W [3]
							words [37], words [38], words [39], words [40], words [41], words [42] = "0xc1f00000", "0xc0a00000", "0xc2200000", "0x42480000", "0x41400000", "0x42700000"
						end
					elseif result == "need_placement" then
						if words [44] ~= ZERO or scene == 0 then stop ("entity_candidate.state", "placement asked of a candidate that cannot need it") end
						words [44], words [23] = "0x3f800000", "0x3f800000"
						if mobile then pose_observe (1) end
					elseif result == "need_normal" then
						if words [24] ~= ZERO or words [44] == ZERO or words [23] == ZERO or mobile then stop ("entity_candidate.state", "normal asked of a candidate that cannot need it") end
						words [24] = "0x3f800000"
						-- a SINGLE fault of the final question's own scene: zero, or a scene no earlier reply named
						if scenario.final_scene == "zero" then scene = 0 elseif scenario.final_scene == "unbound" then scene = 10 end
					elseif result == "buildings" then
						if words [9] == ZERO then stop ("entity_candidate.state", "buildings named for a candidate that is not a city") end
						-- a city's buildings are the host's list: the traversal asks it, and the next scripted answer is the first building's own decision
						if scenario.city then
							local served = city_traverse ()
							-- a city whose own state the campaign reports: each building's own dialogue is its own offer (the shared C offers it under its place in the city's list: `of_city`, its flags and facts not read yet), so the
							-- questions that follow name the building the traversal served and never the city
							if scenario.city_state then
								child_name = served
								for _, index in ipairs ({ 8, 9, 12, 14, 15, 16, 37, 38, 39, 40, 41, 42, 43 }) do words [index] = ZERO end
								words [11] = "0x3f800000"
							end
						else
							return false
						end
					else
						local count = 0
						if type (answer.p) == "string" then for _ in string.gmatch (answer.p, "[^,]+") do count = count + 1 end end
						if count ~= 12 and count ~= 16 then stop ("entity_candidate.pose", "pose of " .. count) end
						if scene == 0 then stop ("entity_candidate.state", "a narrow phase for a candidate that has no scene") end
						for token in string.gmatch (answer.p, "[^,]+") do
							if math.floor (tonumber (string.sub (token, 3), 16) / 8388608) % 256 == 255 then stop ("entity_candidate.pose", "a pose word is not a finite number") end
						end
						return true
					end
				end
				stop ("entity_candidate.state", "the dialogue did not end")
			end
			-- the same candidate offered again by a later scan (an unchanged city is asked about on every frame a segment reaches its sector): the dialogue restarts from the first scripted answer
			local function candidate_dialogue ()
				local narrow = candidate_dialogue_once ()
				if scenario.city_state_again then narrow = candidate_dialogue_once () end
				return narrow
			end
			-- the campaign's driver of the enumeration (mb_hostscan.h) as the shared C applies it: it asks the host, serves the sector (0, 0) with ONE entry, offers it, reports the result and ends when the host does. The
			-- legality of each state against the phase and the cursor's grammar are the exchange's rules; the stop text is the real wrapper. `offer` runs the candidate dialogue and the narrow phase of ONE entry
			local scan_altering = { scan_none = true, withheld_scan = true, bad_scan_state = true, bad_scan_cursor = true, withdrawn_backing = true, withdrawn_child = true, withdrawn_box_state = true }
			-- the campaign's roster events (mb_hostplacemap.h): a sector list's insertion (phase 4: position, entity type and side length, and a hosted mobile's identity), removal (5: the key in the identity place)
			-- and the boot (6). All other facts are zero. The scripted sector holds ONE entry: the first walk is a boot and an insertion, every later walk a relink (a removal and a new insertion with a new key). The host's
			-- own answer comes back unaltered in every mode; the reply is checked as the shared C checks it (the state each phase allows, a key only for an insertion)
			local ROSTER_PHASES = { insert = "0x40800000", remove = "0x40a00000", reset = "0x40c00000" }
			local function roster_event (kind, keep)
				local words = {}
				for index = 1, 16 do words [index] = ZERO end
				words [12] = ROSTER_PHASES [kind]
				local identity = ""
				if kind == "insert" then
					-- the entity type the insertion states: an object (17), or a CITY (5) when the scenario's placement is a city whose own state the campaign reports
					words [1], words [2], words [3], words [4], words [7] = OLD_W [1], OLD_W [2], OLD_W [3], (scenario.city_state or scenario.city_box) and "0x40a00000" or "0x41880000", "0x447a0000"
					if scenario.roster_mobile then identity = "eech-world:1" end
				elseif kind == "remove" then
					identity = roster_key
				end
				local fault = scenario.bad_roster_request
				if fault == "phase7" then words [12] = "0x41880000"
				elseif fault == "insert_stray_word" and kind == "insert" then words [8] = "0x3f800000"
				elseif fault == "remove_no_key" and kind == "remove" then identity = ""
				elseif fault == "remove_text" and kind == "remove" then identity = "eech-world:1"
				elseif fault == "reset_identity" and kind == "reset" then identity = "5"
				elseif fault == "insert_long_identity" and kind == "insert" then identity = string.rep ("x", 129) end
				local key = table.concat (words, ",") .. (identity ~= "" and (" " .. identity) or "")
				if fault == "trailing_space" and kind == "insert" then key = key .. " " end
				local ok, answer = ask_model ("entity_scan", 0, nil, nil, nil, key, true)
				if not ok then stop ("entity_scan.callback", tostring (answer)) end
				local wanted = ({ insert = "placed", remove = "removed", reset = "cleared" }) [kind]
				if type (answer) ~= "table" or answer.result ~= wanted then stop ("entity_scan.state", "a roster event of phase " .. kind .. " allows only " .. wanted) end
				if kind == "insert" and not keep then roster_key = answer.name end
				return answer
			end
			-- a CITY event (the buildings of a city: phases 7 to 12): every fact is zero but the phase and an insertion's entity type; the identity place holds the keys. The host's answer comes back unaltered; a refusal
			-- is the bridge's own fatal with the host's reason
			local CITY_PHASES = { child_insert = "0x40e00000", child_remove = "0x41000000", child_first = "0x41100000", child_next = "0x41200000", link = "0x41300000", name = "0x41400000" }
			local function city_event (kind, identity)
				local words = {}
				for index = 1, 16 do words [index] = ZERO end
				words [12] = CITY_PHASES [kind]
				if kind == "child_insert" then words [4] = (scenario.city_box or scenario.building_state) and "0x40c00000" or "0x41a80000" end
				local fault = scenario.bad_city_request
				if fault == "stray_fact" then words [6] = "0x3f800000"
				elseif fault == "no_city" and kind == "child_first" then identity = ""
				elseif fault == "two_keys_first" and kind == "child_first" then identity = identity .. " 1"
				elseif fault == "one_key_next" and kind == "child_next" then identity = string.match (identity, "^(%d+)")
				elseif fault == "bad_key" and kind == "child_next" then identity = string.gsub (identity, "^(%d+)", "0%1")
				elseif fault == "name_empty" and kind == "name" then identity = string.match (identity, "^(%d+)") .. " "
				elseif fault == "name_control" and kind == "name" then identity = identity .. "\n"
				elseif fault == "next_not_last_served" and kind == "child_next" then identity = string.gsub (identity, " (%d+)$", function (child) return " " .. tostring (tonumber (child) + 1) end) end
				local key = table.concat (words, ",") .. (identity ~= "" and (" " .. identity) or "")
				local ok, answer = ask_model ("entity_scan", 0, nil, nil, nil, key, not scan_altering [MODE])
				if not ok then stop ("entity_scan.callback", tostring (answer)) end
				if type (answer) == "table" and answer.result == "refused" then stop ("entity_scan.refused", tostring (answer.reason)) end
				if type (answer) ~= "table" then stop ("entity_scan.answer", "result") end
				return answer
			end
			city_traverse = function ()
				local list = city_list or roster_key
				local at = city_event ("child_first", list).name
				-- a city whose own state the campaign reports: the first building the host serves is offered at once (the caller's own attempt hits it and ends the traversal), under its place in the city's list
				if scenario.city_state then return at end
				-- (a building the caller's own collision attempt hits ends the traversal before it asks for the next building)
				if scenario.city_first_only then return end
				while at ~= nil and at ~= "" do
					-- (the building's own candidate dialogue is the next scripted answer of the walk)
					at = city_event ("child_next", list .. " " .. at).name
				end
			end
			-- the FIXED_STATE report of the placement just inserted (phase 13): its position, the entity type the insertion stated, the heading, the shape and the fixed sub type (the host derives the collision-fixed and align flags from it), under the placement's key
			-- (mb_hostentity.h, campaign_entity_fixed_exchange). The host's answer comes back unaltered; a refusal is the bridge's own fatal with the host's reason
			local function fixed_report (key_of)
				local words = {}
				for index = 1, 16 do words [index] = ZERO end
				words [1], words [2], words [3] = OLD_W [1], OLD_W [2], OLD_W [3]
				words [4], words [6], words [11], words [12] = "0x41880000", "0x41100000", "0x41400000", "0x41500000"
				local fault = scenario.bad_fixed_request
				if fault == "stray_fact" then words [7] = "0x3f800000"
				elseif fault == "withdraw_word" then words [10] = "0x3f800000"
				elseif fault == "stale_flag" then words [8] = "0x3f800000" end
				local identity = key_of
				if fault == "bad_key" then identity = "01"
				elseif fault == "no_key" then identity = "" end
				local ok, answer = ask_model ("entity_scan", 0, nil, nil, nil, table.concat (words, ",") .. (identity ~= "" and (" " .. identity) or ""), true)
				if not ok then stop ("entity_scan.callback", tostring (answer)) end
				if type (answer) == "table" and answer.result == "refused" then stop ("entity_scan.refused", tostring (answer.reason)) end
				if type (answer) ~= "table" or answer.result ~= "placed" then stop ("entity_scan.state", "a fixed state report allows only placed") end
				return answer
			end
			-- the CITY_STATE report of a city placement (phase 14): its position, box minimum and box maximum (facts 1 to 9) and the phase, under the placement's key (mb_hostentity.h). The host's answer comes back unaltered
			local function city_state_report (key_of)
				local words = {}
				for index = 1, 16 do words [index] = ZERO end
				words [1], words [2], words [3] = OLD_W [1], OLD_W [2], OLD_W [3]
				words [4], words [5], words [6], words [7], words [8], words [9], words [11], words [12] = "0xc1f00000", "0xc0a00000", "0xc2200000", "0x42480000", "0x41400000", "0x42700000", "0x40800000", "0x41600000"
				local fault = scenario.bad_city_state_request
				if fault == "stray_fact" then words [10] = "0x3f800000"
				elseif fault == "word_13" then words [14] = "0x3f800000" end
				local identity = key_of
				if fault == "bad_key" then identity = "01"
				elseif fault == "no_key" then identity = "" end
				local ok, answer = ask_model ("entity_scan", 0, nil, nil, nil, table.concat (words, ",") .. (identity ~= "" and (" " .. identity) or ""), true)
				if not ok then stop ("entity_scan.callback", tostring (answer)) end
				if type (answer) == "table" and answer.result == "refused" then stop ("entity_scan.refused", tostring (answer.reason)) end
				if type (answer) ~= "table" or answer.result ~= "placed" then stop ("entity_scan.state", "a city state report allows only placed") end
				return answer
			end
			-- the city's own state at its insertion with the original's INVERTED initial box (minimum 1.0e6, maximum -1.0e6): the statement a city makes before its first building links
			local function sentinel_city_report (key_of)
				local words = {}
				for index = 1, 16 do words [index] = ZERO end
				words [1], words [2], words [3] = OLD_W [1], OLD_W [2], OLD_W [3]
				words [4], words [5], words [6], words [7], words [8], words [9], words [11], words [12] = "0x49742400", "0x49742400", "0x49742400", "0xc9742400", "0xc9742400", "0xc9742400", "0x40800000", "0x41600000"
				local ok, answer = ask_model ("entity_scan", 0, nil, nil, nil, table.concat (words, ",") .. " " .. key_of, true)
				if not ok then stop ("entity_scan.callback", tostring (answer)) end
				if type (answer) == "table" and answer.result == "refused" then stop ("entity_scan.refused", tostring (answer.reason)) end
				if type (answer) ~= "table" or answer.result ~= "placed" then stop ("entity_scan.state", "a city state report allows only placed") end
			end
			-- the FIXED_STATE report of a building listed in a city (phase 13), under its place in the city's list: a city building (6), its heading, the shape of the scenario, the sub type CITY_BUILDING
			local function building_report (child)
				local words = {}
				for index = 1, 16 do words [index] = ZERO end
				words [1], words [2], words [3] = OLD_W [1], OLD_W [2], OLD_W [3]
				words [4], words [5], words [6], words [11], words [12] = "0x40c00000", "0x3f000000", scenario.box_shape or "0x40a00000", "0x40a00000", "0x41500000"
				local ok, answer = ask_model ("entity_scan", 0, nil, nil, nil, table.concat (words, ",") .. " " .. child, true)
				if not ok then stop ("entity_scan.callback", tostring (answer)) end
				if type (answer) == "table" and answer.result == "refused" then stop ("entity_scan.refused", tostring (answer.reason)) end
				if type (answer) ~= "table" or answer.result ~= "placed" then stop ("entity_scan.state", "a fixed state report allows only placed") end
			end
			-- a building's link asks the host to build its city's box (phase 15: nothing stated but the phase; the identity place is the city's sector key, one space and the building's key). The reply is checked as the shared
			-- C checks it before any word would be applied (mb_hostentity.h): `expanded`, the city's key, six finite patterns. The host's answer comes back unaltered; a refusal is the bridge's own fatal with the host's reason
			local function city_expand_request (city_key, child)
				local words = {}
				for index = 1, 16 do words [index] = ZERO end
				words [12] = "0x41700000"
				local fault = scenario.bad_expand_request
				local identity = city_key .. " " .. child
				if fault == "stray_fact" then words [6] = "0x3f800000"
				elseif fault == "one_key" then identity = city_key
				elseif fault == "bad_key" then identity = city_key .. " 0" .. child end
				local ok, answer = ask_model ("entity_scan", 0, nil, nil, nil, table.concat (words, ",") .. " " .. identity, not scan_altering [MODE])
				if not ok then stop ("entity_scan.callback", tostring (answer)) end
				if type (answer) == "table" and answer.result == "refused" then stop ("entity_scan.refused", tostring (answer.reason)) end
				if type (answer) ~= "table" or answer.result ~= "expanded" or answer.name ~= city_key or type (answer.p) ~= "string" then stop ("entity_scan.state", "a city expansion allows only expanded, with the city's key and a box") end
				EXPANDED [#EXPANDED + 1] = answer.p
				return answer
			end
			-- the campaign's cities at BOOT: the city is placed and states its inverted box, and each building is listed, states its fixed state and links (the link's question follows)
			local function boot_city_box ()
				roster_event ("reset")
				roster_event ("insert")
				sentinel_city_report (roster_key)
				local layout = scenario.city_box_layout
				-- a SECOND live city with its own acknowledged state, a building of each, and the expansion asked for the pair that does not belong together: the building of the FIRST city under the SECOND city's key
				if layout == "cross" or layout == "cross_relinked" then
					local second = roster_event ("insert", true).name
					sentinel_city_report (second)
					local first_child = city_event ("child_insert", roster_key).name
					building_report (first_child)
					local second_child = city_event ("child_insert", second).name
					building_report (second_child)
					if layout == "cross" then city_expand_request (second, first_child) return end
					-- the first city's sector placement is relinked while its buildings stay listed (its list keeps the first key's name): the new placement continues the list, and is asked about the OTHER city's building
					local anchor = roster_key
					roster_event ("remove")
					roster_event ("insert")
					city_event ("link", roster_key .. " " .. anchor)
					sentinel_city_report (roster_key)
					city_expand_request (roster_key, second_child)
					return
				end
				-- the same pair after a legitimate relink: the building is listed under the key the first placement named the list by, and the city now stands in a new placement that continues that list
				if layout == "relink" then
					local child = city_event ("child_insert", roster_key).name
					building_report (child)
					local anchor = roster_key
					roster_event ("remove")
					roster_event ("insert")
					city_event ("link", roster_key .. " " .. anchor)
					sentinel_city_report (roster_key)
					city_expand_request (roster_key, child)
					return
				end
				-- an EMPTY city's relink is a removal and a new insertion with no list to continue: the first building is listed under the new placement
				if layout == "empty_relink" then
					roster_event ("remove")
					roster_event ("insert")
					sentinel_city_report (roster_key)
					local child = city_event ("child_insert", roster_key).name
					building_report (child)
					city_expand_request (roster_key, child)
					return
				end
				-- a boot refusal that is NOT the withdrawn building's: it stops at the same generic marker before any expansion, with nothing withdrawn
				if layout == "unrelated_refusal" then
					local child = city_event ("child_insert", roster_key).name
					building_report (child)
					stop ("entity_scan.refused", "[roster.unrelated] an unrelated refusal at boot")
				end
				for _ = 1, scenario.city_box do
					local child = city_event ("child_insert", roster_key).name
					city_children [#city_children + 1] = child
					building_report (child)
					city_expand_request (roster_key, child)
				end
			end
			local function scan_walk (offer)
				if scenario.no_scan_questions then offer () return end
				for index = #city_children, 1, -1 do city_event ("child_remove", (city_list or roster_key) .. " " .. city_children [index]) end
				city_children, city_list = {}, nil
				roster_event (roster_key == nil and "reset" or "remove")
				roster_event ("insert")
				if scenario.fixed_state then fixed_report (roster_key) end
				if scenario.city_state then city_state_report (roster_key) end
				-- a SECOND live placement the real provider also holds a state for (stated after the served one; the scan serves the first): a candidate that names it names a placement the host did not serve
				if (scenario.fixed_state or scenario.city_state) and scenario.second_held then
					scenario.second_key = roster_event ("insert", true).name
					if scenario.city_state then city_state_report (scenario.second_key) else fixed_report (scenario.second_key) end
				end
				for _ = 1, scenario.city or 0 do
					city_children [#city_children + 1] = city_event ("child_insert", roster_key).name
					-- the campaign states the fixed state of the building it just listed (sub type CITY_BUILDING, which collides), under the building's own place in the city's list
					if scenario.building_state then building_report (city_children [#city_children]) end
					if scenario.city_state then city_state_report (roster_key) end
				end
				if scenario.city_relink and #city_children > 0 then
					-- the city's sector placement is relinked while its buildings stay listed (the fixed-position setter every city shares): it is removed and inserted again, and the new placement is declared to
					-- continue the list the buildings were listed under, whose name does not change
					city_list = roster_key
					roster_event ("remove")
					roster_event ("insert")
					city_event ("link", roster_key .. " " .. city_list)
					if scenario.city_state then city_state_report (roster_key) end
				end
				if scenario.city_cycle and #city_children > 0 then
					-- the last building leaves while the city stays placed, a building is listed again under the list's name BEFORE the city moves, and the city is relinked again: the name must still be the record's
					for index = #city_children, 1, -1 do city_event ("child_remove", city_list .. " " .. city_children [index]) end
					city_children = { city_event ("child_insert", city_list).name }
					roster_event ("remove")
					roster_event ("insert")
					city_event ("link", roster_key .. " " .. city_list)
					if scenario.city_state then city_state_report (roster_key) end
				end
				if scenario.city_link and #city_children > 0 then
					-- the first building is also placed in its sector, declared the same entity as its place in the city's list, and named by an object identity issued for it
					local sector = roster_event ("insert", true).name
					city_event ("link", sector .. " " .. city_children [1])
					city_event ("name", sector .. " " .. (scenario.city_object_length and string.rep ("y", scenario.city_object_length) or "eech-world:object:7"))
				end
				-- only the roster's events reached the host: no scan question was asked
				if scenario.only_roster_events then offer () return end
				local words = {}
				for index = 1, 16 do words [index] = ZERO end
				words [1], words [2], words [3] = OLD_W [1], OLD_W [2], OLD_W [3]
				words [4], words [5], words [6] = NEW_W [1], NEW_W [2], NEW_W [3]
				words [7], words [10], words [11] = "0x42c80000", "0x43c78000", "0x43c78000"
				local PHASES = { begin = ZERO, entry = "0x3f800000", ["end"] = "0x40000000", resolved = "0x40400000" }
				-- the key of the entry the scan stands at rides after the 16 facts in ENTRY and RESOLVED only (mb_hostentity.h)
				local function ask (phase, excluded, hit, standing)
					words [12], words [15], words [16] = PHASES [phase], excluded and "0x3f800000" or ZERO, hit and "0x3f800000" or ZERO
					local key = table.concat (words, ",")
					local fault = scenario.bad_scan_request
					if fault == "count" then key = string.match (key, "^(.*),[^,]+$") end
					if (phase == "entry" or phase == "resolved") and fault ~= "no_key" then
						if fault == "bad_key" then standing = "01"
						elseif fault == "other_key" then standing = tostring (tonumber (standing) + 1)
						elseif fault == "ended_key" then standing = ROSTER.ended or "1" end
						key = key .. " " .. standing
					elseif phase == "begin" and fault == "begin_key" then key = key .. " " .. roster_key end
					local ok, answer = ask_model ("entity_scan", 0, nil, nil, nil, key, not scan_altering [MODE])
					if not ok then stop ("entity_scan.callback", tostring (answer)) end
					-- a refusal of the host is the bridge's own fatal, with the host's reason (as for the candidate)
					if type (answer) == "table" and answer.result == "refused" then stop ("entity_scan.refused", tostring (answer.reason)) end
					if type (answer) ~= "table" or not ({ first = true, next = true, offer = true, done_none = true, done_hit = true }) [answer.result] then stop ("entity_scan.answer", "result") end
					return answer
				end
				local function allowed (answer, ...)
					for _, state in ipairs ({ ... }) do if answer.result == state then return end end
					stop ("entity_scan.state", "state " .. answer.result .. " is not allowed here")
				end
				local answer, standing
				if scenario.scan_from_entry then
					-- a scan the wrapper first sees mid-way (armed between two of its questions): it stands at the key of the entry placed first, or, for `ended`, at that key after the entry was relinked under a new one
					standing = roster_key
					if scenario.scan_from_entry == "ended" then roster_event ("remove") roster_event ("insert") end
				else
					answer = ask ("begin", false, false)
					allowed (answer, "first", "done_none")
					if answer.result == "done_none" then
						-- a (wrong) campaign that offers an entry although the host ended the scan
						if scenario.ignore_none then offer () end
						-- a campaign that observes a mobile although the host ended the scan (no candidate question follows): the observation alone must fail the empty-enumeration variant
						if scenario.pose_after_none then pose_observe (0) end
						return
					end
					-- the cursor: two whole numbers inside the map; the scripted map has the one sector (0, 0), so only zero words are valid (a fractional or other word is the exchange's or the campaign's cursor refusal)
					local cursor = {}
					for token in string.gmatch (answer.p or "", "[^,]+") do cursor [#cursor + 1] = token end
					if #cursor ~= 2 then stop ("entity_scan.cursor", "two patterns") end
					if cursor [1] ~= ZERO or cursor [2] ~= ZERO then stop ("entity_scan.cursor", "a sector that is not a whole number inside the map") end
					words [13], words [14] = cursor [1], cursor [2]
					-- the entry the host served: the scan stands at it until the host serves the next one
					standing = answer.name
				end
				answer = ask ("entry", false, false, standing)
				allowed (answer, "offer")
				last_hit = false
				offer ()
				answer = ask ("resolved", false, last_hit, standing)
				allowed (answer, last_hit and "done_hit" or "next")
				if answer.result == "done_hit" then return end
				-- a scan seen mid-way is seen no further: its end of the list is a valid question the refusal mode would legitimately alter, which would hide whether the earlier questions were
				if scenario.scan_from_entry then return end
				answer = ask ("end", false, false)
				allowed (answer, "first", "done_none")
			end
			-- the scripted flight's entity lookup: the campaign's own rules applied as far as the driver's bookkeeping needs (the bridge's shape checks, the C domain checks); the stop text is the real wrapper
			local function entity_test ()
				tests = tests + 1
				-- the ORIGINAL profile's flight: the narrow phase runs inside the module and the host is asked nothing
				if scenario.original_tests then
					if scenario.original_hits then hits = hits + 1 end
					return
				end
				if scenario.no_entity_questions then return end
				for _ = 1, scenario.questions or 1 do
					if scenario.engine_stop and scenario.engine_stop.after == since_launch then
						if scenario.engine_stop.text_only then error ("a Lua error that merely mentions " .. "[model-provider:" .. scenario.engine_stop.marker .. "] in its text", 0) end
						error (scenario.engine_stop.wrapper or (FATAL .. "[model-provider:" .. scenario.engine_stop.marker .. "] scripted"), 0)
					end
					scan_walk (function ()
					if candidate_dialogue () then
					local key, extra = pose_key (scenario.moving)
					local ok, answer = ask_model ("entity_collision", 7, extra, nil, nil, key, MODE == "control")
					if not ok then stop ("entity_collision.callback", tostring (answer)) end
					local words = {}
					if type (answer) == "table" and type (answer.p) == "string" then for token in string.gmatch (answer.p, "[^,]+") do words [#words + 1] = token end end
					ENTITY_LOG [#ENTITY_LOG + 1] = { key = key, result = type (answer) == "table" and answer.result or "?", mask = type (answer) == "table" and answer.mask or -1, p = type (answer) == "table" and answer.p or "" }
					if type (answer) ~= "table" or (answer.result ~= "hit" and answer.result ~= "miss") then stop ("entity_collision.answer", "result") end
					local mask = answer.mask
					if type (mask) ~= "number" or mask ~= mask or mask ~= math.floor (mask) or mask < 0 or mask > 63 then stop ("entity_collision.answer", "mask must be a whole number within 0..63") end
					if #words ~= 6 then stop ("entity_collision.answer", "p") end
					if answer.name ~= "CANDIDATE" then stop ("entity_collision.identity", "name") end
					local point, normal = mask % 8, math.floor (mask / 8)
					if not scenario.lenient_engine and ((point ~= 0 and point ~= 7) or (normal ~= 0 and normal ~= 7) or (point ~= 0 and normal == 0) or (answer.result == "hit" and mask ~= 63)) then stop ("entity_collision.write_mask", "mask " .. mask) end
					for index = 1, 6 do
						local assigned = (index <= 3 and point ~= 0) or (index > 3 and normal ~= 0)
						if not scenario.lenient_engine and not assigned and words [index] ~= ZERO then stop ("entity_collision.unwritten", "word " .. index) end
					end
					if answer.result == "hit" and not scenario.hit_ignored then hits = hits + 1 flight_hit = true last_hit = true end
					if answer.result == "miss" and scenario.hits_despite_miss then hits = hits + 1 end
					elseif scenario.ignore_skip then
						-- a (wrong) campaign that asks the narrow phase although the host skipped the candidate
						ask_model ("entity_collision", 7, nil, nil, nil, pose_key (false), false)
					end
					end)
				end
			end
			function engine:frame (milliseconds)
				-- a natural weapon's offer before the case's launch (boot, warmup): the witness classifies it too, and never alters it
				if scenario.pose_before_launch and not launched then pose_observe (0, true) end
				if not (launched and scenario.stall_after_launch) then now = now + (scenario.clock_step or milliseconds / 1000) end
				frame_no = frame_no + 1
				if frame_no <= (scenario.engagement_until or 1000000) then
					for _, engagement in ipairs (scenario.engagements or {}) do
						local target_id = "eech-world:" .. engagement.target
						local request = {
							mode = "vectors", echo = 1, source_host_id = "eech-world:" .. engagement.source, target_host_id = target_id,
							weapon = { weapon = engagement.weapon, present = true }, target = { position = { 1, 2, 3 }, motion_vector = (scenario.motion or {}) [target_id] or { 0, 0, 0 }, mobile = true },
						}
						local pristine = deep_copy (request)
						VECTORS_SEEN = nil
						config.aim (request)
						VECTORS_ASKED = VECTORS_ASKED + 1
						local changed = {}
						differences (pristine, request, "", changed)
						-- the witness forwards the SAME request, unchanged, to the host's own aim
						if VECTORS_SEEN ~= request or #changed > 0 then VECTORS_ALTERED = true end
					end
				end
				weapon_step ("weapon_step", false)
				local effective = weapon_step ("weapon_step", true)
				weapon_step ("weapon_birth", true)
				if launched and effective then precision = precision + 1 end
				-- other operations the host answers itself, every frame, before and after the launch
				if not scenario.no_other_ops then
					ask_model ("scene", 3, nil, nil, nil, nil, true)
					ask_model ("triangle", 3, 0, nil, nil, nil, true)
				end
				if launched then
					since_launch = since_launch + 1
					if effective and not scenario.no_entity_tests then entity_test () end
					local lines = ((flight_hit and scenario.after_hit) or scenario.after or {}) [since_launch] or {}
					for _, line in ipairs (lines) do queue [#queue + 1] = line end
				end
				for _, index in ipairs ((scenario.no_query_frame_steps and #MQ_CALLS > 0) and {} or scenario.steps or {}) do
					local ground = scenario.ground or 10
					-- the speed demand the engine gives this step: the scenario's per identity, replaced in the controlled query's one frame by `query_demand` ('missing' is no desired_velocity at all)
					local demand = (scenario.demand or {}) [index]
					if #MQ_CALLS > 0 and (scenario.query_demand or {}) [index] ~= nil then
						demand = scenario.query_demand [index]
						if demand == "missing" then demand = nil end
					end
					local step = { host_id = "eech-world:" .. index, mode = "move", terrain = { elevation = ground, forest = false, face_normal = { 0, 1, 0 } }, map_max = { 1000, scenario.ceiling or 2000, 1000 },
						desired_velocity = demand, max_acceleration = 3, delta_seconds = 0.5, waypoint = { 100, 0, 100 }, map_min = { 0, 0, 0 }, tacview_logging = false }
					CURRENT_STEP = step
					-- a routed step the core gives no terrain (the supported None input): always for `bare_steps`, or only in the controlled query's one frame (after its prepare) for `bare_in_query_frame`
					local bare = (scenario.bare_steps or {}) [index] or ((scenario.bare_in_query_frame or {}) [index] and #MQ_CALLS > 0)
					if bare then step.terrain = nil end
					config.move_vehicle (step)
					-- the engine's own step table is never mutated by the host-movement wrapper
					if (bare and step.terrain ~= nil) or (not bare and step.terrain.elevation ~= ground) or (step.desired_velocity ~= demand and not (demand ~= demand and step.desired_velocity ~= step.desired_velocity)) then CALLER_MUTATED = true end
				end
			end
			function engine:wpn_case_query (id, weapon)
				local entry = by_id [id]
				local position = entry and entry.position or "1000000,0,1000000"
				if weapon == 0 then return "alive=1 weapon_count=0 has_target=0 position=" .. position end
				if entry and armed_weapons [entry.index] and weapon == 7 then return "alive=1 weapon_count=3 warhead=2 guidance=0 aiming=2 decoy=0 position=" .. position end
				return "alive=1 weapon_count=0 warhead=2 guidance=0 aiming=2 decoy=0 position=" .. position
			end
			function engine:wpn_case_launch (launcher, target, weapon)
				launched = true
				LAUNCH = { launcher = launcher, target = target, weapon = weapon }
				for _, line in ipairs (scenario.births or { "kind=birth w=5 type=gun" }) do queue [#queue + 1] = line end
				-- a launch whose target is already the launcher's link sets no link (the fixture's own rule): link_before=1 and restored=-1
				local already = linked [launcher] == target
				return scenario.launch_report or ("wpn-case-launch launcher=helicopter weapon=7 target_link_before=" .. (already and 1 or 0) .. " target_link=1 count_before=3 count_after=2 births=" .. #(scenario.births or { 1 }) .. " creates=1 restored=" .. (already and -1 or 1))
			end
			local latched_query
			local function segment_text (k)
				local base = 0x41000000 + k * 0x100
				return string.format ("0x%08x,0x%08x,0x%08x", base, base + 1, base + 2), string.format ("0x%08x,0x%08x,0x%08x", base + 16, base + 17, base + 18)
			end
			if not scenario.no_query_surface then
				function engine:wpn_moving_query_prepare (id)
					MQ_CALLS [#MQ_CALLS + 1] = "prepare@" .. now .. ":" .. id
					-- the REAL bridge delivers the named, non-poison refusal as `eech: bad argument: <the case's report>` (eech-dc lua_error over keep_case_reason)
					if scenario.query_refuse == "prepare" then error ("eech: bad argument: wpn-case-error reason=target_model_unsupported host_id=" .. mq_enc (id), 0) end
					if scenario.query_refuse == "prepare_lost" then error ("eech: bad argument: rejected by the engine", 0) end
					latched_query = id
					local lines = { "wpn-moving-query-prepare host_id=" .. mq_enc (id) .. " segments=3 target_type=ENTITY_TYPE_ROUTED_VEHICLE radius=0x41200000" }
					for k = 0, 2 do
						local a, b = segment_text (k)
						lines [#lines + 1] = "wpn-moving-query-segment seg=" .. k .. " bearing=" .. MQ_BEARING_WORDS [k + 1] .. " a=" .. a .. " b=" .. b
					end
					local text = table.concat (lines, "\n") .. "\n"
					if scenario.query_fault and string.sub (scenario.query_fault, 1, 8) == "prepare_" then text = mq_fault (scenario.query_fault, text) end
					return text
				end
				function engine:wpn_moving_query_run ()
					MQ_CALLS [#MQ_CALLS + 1] = "run@" .. now
					if scenario.query_refuse == "run" then error ("eech: bad argument: wpn-case-error reason=target_not_alive host_id=" .. mq_enc (tostring (latched_query)), 0) end
					local lines = { "wpn-moving-query-run host_id=" .. mq_enc (tostring (latched_query)) .. " segments=3" }
					-- the scripted physics: the rays answer from the demand the host's solver was last given for the target (only when the scenario defines the zero-demand answer)
					local rays = (scenario.query_zero_demand_hits and LAST_DEMAND [tostring (latched_query)] == 0) and scenario.query_zero_demand_hits or scenario.query_hits or { 1, 0, 1 }
					for k = 0, 2 do
						-- the production caller's questions about the target: the hosted module asks the host, the original asks nothing
						if not scenario.query_original then scan_walk (function () candidate_dialogue () end) end
						local a, b = segment_text (k)
						if scenario.query_segment_changed and k == 2 then a = "0x41000000,0x41000001,0x41000003" end
						local head = "wpn-moving-query-result seg=" .. k .. " hit=" .. rays [k + 1] .. " a=" .. a .. " b=" .. b
						if rays [k + 1] == 1 then
							if scenario.query_other_hit_exotic and k == 0 then
								-- a hit of ANOTHER entity, whose roster identity is the exotic maximum one, and a hit entity that has no roster identity
								lines [#lines + 1] = head .. " hit_is_target=0 hit_type=ENTITY_TYPE_HELICOPTER hit_host_id=" .. mq_enc (EXOTIC_ID) .. " point=0xffffffff,0x7fc00001,0x80000000 normal=0x00000000,0x3f800000,0x00000000"
							elseif scenario.query_building_hit and k == 0 then
								lines [#lines + 1] = head .. " hit_is_target=0 hit_type=ENTITY_TYPE_CITY_BUILDING hit_host_id=- point=0x42000000,0x42000001,0x42000002 normal=0x00000000,0x3f800000,0x00000000"
							else
								lines [#lines + 1] = head .. " hit_is_target=1 hit_type=ENTITY_TYPE_ROUTED_VEHICLE hit_host_id=" .. mq_enc (tostring (latched_query)) .. " point=0x42000000,0x42000001,0x42000002 normal=0x00000000,0x3f800000,0x00000000"
							end
						else
							lines [#lines + 1] = head .. " hit_is_target=0 hit_type=none hit_host_id=- point=- normal=-"
						end
					end
					latched_query = nil
					local text = table.concat (lines, "\n") .. "\n"
					if scenario.query_fault and string.sub (scenario.query_fault, 1, 4) == "run_" then text = mq_fault (scenario.query_fault, text) end
					return text
				end
			end
			function engine:wpn_events ()
				local text = table.concat (queue, "\n")
				queue = {}
				return text
			end
			function engine:wpn_trace ()
				return "wpn-counter name=other value=9\nwpn-counter name=contact_any_tests value=" .. tests .. "\nwpn-counter name=contact_any_hits value=" .. hits
					.. "\nwpn-counter name=precision_high value=" .. precision .. "\n"
			end
			-- the campaign's cities are made at boot, before any weapon window (the host's roster is as the real boot would have it: boot, the city, its buildings)
			if scenario.city_box then boot_city_box () end
			return engine
		end,
	}
end
"#;

struct Driven {
    result: Result<(), String>,
    rows: String,
    closed: i64,
    created: bool,
    /// (mode, armed at the call, paths where the host's aim differed from the engine's pristine request, the printed first word of the unrelated answer)
    aim_log: Vec<(String, bool, String, String)>,
    caller_mutated: bool,
    /// the operations whose answer or arguments did not come back exactly as the host's own
    delegation_broken: Vec<String>,
    /// (the request's pose text, the answer's result, mask and words) of every entity question the scripted engine asked
    entity_log: Vec<(String, String, i64, String)>,
    /// (launcher, target, weapon) of the launch the driver made
    launch: Option<(String, String, i64)>,
    /// (scene, result, facts text) of every candidate question the REAL provider answered
    candidate_log: Vec<(i64, String, String)>,
    /// (scene, facts text, result, refusal reason or empty) of EVERY candidate call the scripted provider received, in order: the campaign's questions and the host's own extra evaluations alike
    candidate_calls: Vec<(i64, String, String, String)>,
    /// (phase pattern, result) of every scan question the REAL provider answered
    scan_log: Vec<(String, String)>,
    /// the identity place (the key of the entry the scan stands at) every scan question carried, in the order they were asked
    scan_standing: Vec<String>,
    /// (phase pattern, result, the key the host named, the identity place) of every roster event the REAL provider answered
    roster_log: Vec<(String, String, String, String)>,
    /// (phase pattern, result, the key the host named, the identity place) of every city event the REAL provider answered
    city_log: Vec<(String, String, String, String)>,
    /// (phase pattern, result, the key the host named, the identity place) of every fixed state report the REAL provider answered
    fixed_log: Vec<(String, String, String, String)>,
    /// (phase pattern, result, the city key, the identity place) of every city expansion question the scripted provider answered
    expand_log: Vec<(String, String, String, String)>,
    /// (stage, entity type, alive, result, key) of every pose observation the REAL provider answered
    pose_log: Vec<(i64, i64, i64, String, String)>,
    /// (identity, mode, terrain elevation or -1, after the launch) of every step the host's movement callback was given
    move_log: Vec<(String, String, f64, bool)>,
    /// the `vectors` aim requests the scripted campaign made, and whether any was not forwarded as the very same unchanged request
    vectors: (i64, bool),
    /// the controlled query surface's calls, in order, each with the simulated time it was made at
    query_calls: Vec<String>,
    /// every `move` step the host received as `identity|demand received|phase|fields differing from the engine's own step|demand the engine gave` (see the collaborators)
    demand_log: Vec<String>,
}

/// Runs the real driver once. `scenario` is the inside of the Lua table the collaborators read; `args` are the driver's own arguments.
fn drive(scenario: &str, args: &[(&str, &str)]) -> Driven {
    let lua = Lua::new();
    let mut table = String::from("{ root = 'unused-root', out = 'unused-out', warmup = '1', settle = '2', frame_ms = '1000'");
    for (key, value) in args {
        table.push_str(&format!(", {key} = '{value}'"));
    }
    table.push_str(" }");
    lua.load(format!("ARGS = {table}\nSCENARIO = {{ {scenario} }}"))
        .exec()
        .expect("the scenario is valid Lua");
    // the REAL model provider with its own sector roster, for the scenarios that ask for it (`real_scan`): the scripted collaborators then forward every entity_scan question to it
    let real_host = lua.create_table().expect("a table");
    real_host.set("args", lua.create_table().expect("a table")).expect("args");
    eech_world::model_host::register(&lua, &real_host).expect("the real provider registers");
    lua.globals().set("REAL_HOST", real_host).expect("the real host");
    let roster = std::rc::Rc::new(std::cell::RefCell::new(eech_world::roster::SectorRoster::default()));
    lua.set_app_data(eech_world::roster::SharedRoster(roster.clone()));
    // what the REAL roster holds of the fixed collision state of the sector placement a key names (a read, never a write): the scripted candidate states a withdrawn state in the provider's own words
    let hold = lua
        .create_function(move |_, (key, child): (String, Option<bool>)| {
            use eech_world::roster::FixedHold;
            Ok(
                match eech_world::roster::parse_key(&key).map(|key| roster.borrow().placement_fixed(key, child.unwrap_or(false))) {
                    Some(Ok((_, _, FixedHold::Held(state)))) if !state.collision_fixed() => "no_collision",
                    Some(Ok((_, _, FixedHold::Held(_)))) => "held",
                    Some(Ok((_, _, held))) if format!("{held:?}").starts_with("City(") => "held",
                    Some(Ok((_, _, FixedHold::Withdrawn))) => "withdrawn",
                    Some(Ok((_, _, FixedHold::Never))) => "never",
                    _ => "unknown",
                },
            )
        })
        .expect("the roster hold probe");
    lua.globals().set("ROSTER_HOLD", hold).expect("the probe");
    lua.load(COLLABORATORS)
        .set_name("scripted collaborators")
        .exec()
        .expect("the collaborators load");
    let result = lua.load(SCRIPT).set_name("entity-authority.lua").exec().map_err(|e| e.to_string());
    let written: Vec<String> = lua.globals().get("WRITTEN").expect("the collected output");
    let globals = lua.globals();
    let log: mlua::Table = globals.get("AIM_LOG").expect("the aim log");
    let broken: mlua::Table = globals.get("DELEGATION_BROKEN").expect("the delegation record");
    let entities: mlua::Table = globals.get("ENTITY_LOG").expect("the entity log");
    let launch: Option<mlua::Table> = globals.get("LAUNCH").expect("the launch");
    let candidates: mlua::Table = globals.get("CAND_LOG").expect("the candidate log");
    let scans: mlua::Table = globals.get("SCAN_LOG").expect("the scan log");
    let roster: mlua::Table = globals.get("ROSTER_LOG").expect("the roster log");
    let city: mlua::Table = globals.get("CITY_LOG").expect("the city log");
    let fixed: mlua::Table = globals.get("FIXED_LOG").expect("the fixed state log");
    let expansions: mlua::Table = globals.get("EXPAND_LOG").expect("the city expansion log");
    let poses: mlua::Table = globals.get("POSE_LOG").expect("the pose log");
    let moves: mlua::Table = globals.get("MOVE_LOG").expect("the move log");
    Driven {
        result,
        rows: written.concat(),
        closed: globals.get("CLOSED").expect("the close count"),
        created: globals.get("CREATED").expect("the creation flag"),
        aim_log: log
            .sequence_values::<mlua::Table>()
            .map(|entry| {
                let entry = entry.expect("a log entry");
                (
                    entry.get::<String>("mode").unwrap(),
                    entry.get::<Option<bool>>("launched").unwrap().unwrap_or(false),
                    entry.get::<String>("paths").unwrap(),
                    entry.get::<String>("zero").unwrap(),
                )
            })
            .collect(),
        caller_mutated: globals.get("CALLER_MUTATED").expect("the mutation flag"),
        delegation_broken: broken.sequence_values::<String>().map(|op| op.unwrap()).collect(),
        entity_log: entities
            .sequence_values::<mlua::Table>()
            .map(|entry| {
                let entry = entry.expect("an entity log entry");
                (
                    entry.get::<String>("key").unwrap(),
                    entry.get::<String>("result").unwrap(),
                    entry.get::<i64>("mask").unwrap(),
                    entry.get::<String>("p").unwrap(),
                )
            })
            .collect(),
        launch: launch.map(|t| (t.get("launcher").unwrap(), t.get("target").unwrap(), t.get("weapon").unwrap())),
        demand_log: globals
            .get::<mlua::Table>("DEMAND_LOG")
            .expect("the demand log")
            .sequence_values::<String>()
            .map(|entry| entry.unwrap())
            .collect(),
        query_calls: globals
            .get::<mlua::Table>("MQ_CALLS")
            .expect("the query call log")
            .sequence_values::<String>()
            .map(|call| call.unwrap())
            .collect(),
        vectors: (
            globals.get("VECTORS_ASKED").expect("the aim request count"),
            globals.get("VECTORS_ALTERED").expect("the aim forwarding flag"),
        ),
        move_log: moves
            .sequence_values::<mlua::Table>()
            .map(|entry| {
                let entry = entry.expect("a move log entry");
                (
                    entry.get::<String>("id").unwrap(),
                    entry.get::<String>("mode").unwrap(),
                    entry.get::<f64>("elevation").unwrap(),
                    entry.get::<bool>("launched").unwrap(),
                )
            })
            .collect(),
        pose_log: poses
            .sequence_values::<mlua::Table>()
            .map(|entry| {
                let entry = entry.expect("a pose log entry");
                (
                    entry.get::<i64>("stage").unwrap(),
                    entry.get::<i64>("entity_type").unwrap(),
                    entry.get::<i64>("alive").unwrap(),
                    entry.get::<String>("result").unwrap(),
                    entry.get::<String>("key").unwrap(),
                )
            })
            .collect(),
        scan_log: scans
            .sequence_values::<mlua::Table>()
            .map(|entry| {
                let entry = entry.expect("a scan log entry");
                (entry.get::<String>("phase").unwrap(), entry.get::<String>("result").unwrap())
            })
            .collect(),
        scan_standing: scans
            .sequence_values::<mlua::Table>()
            .map(|entry| entry.expect("a scan log entry").get::<String>("standing").unwrap())
            .collect(),
        city_log: city
            .sequence_values::<mlua::Table>()
            .map(|entry| {
                let entry = entry.expect("a city log entry");
                (
                    entry.get::<String>("phase").unwrap(),
                    entry.get::<String>("result").unwrap(),
                    entry.get::<String>("name").unwrap(),
                    entry.get::<String>("identity").unwrap(),
                )
            })
            .collect(),
        fixed_log: fixed
            .sequence_values::<mlua::Table>()
            .map(|entry| {
                let entry = entry.expect("a fixed state log entry");
                (
                    entry.get::<String>("phase").unwrap(),
                    entry.get::<String>("result").unwrap(),
                    entry.get::<String>("name").unwrap(),
                    entry.get::<String>("identity").unwrap(),
                )
            })
            .collect(),
        expand_log: expansions
            .sequence_values::<mlua::Table>()
            .map(|entry| {
                let entry = entry.expect("a city expansion log entry");
                (
                    entry.get::<String>("phase").unwrap(),
                    entry.get::<String>("result").unwrap(),
                    entry.get::<String>("name").unwrap(),
                    entry.get::<String>("identity").unwrap(),
                )
            })
            .collect(),
        roster_log: roster
            .sequence_values::<mlua::Table>()
            .map(|entry| {
                let entry = entry.expect("a roster log entry");
                (
                    entry.get::<String>("phase").unwrap(),
                    entry.get::<String>("result").unwrap(),
                    entry.get::<String>("name").unwrap(),
                    entry.get::<String>("identity").unwrap(),
                )
            })
            .collect(),
        candidate_calls: globals
            .get::<mlua::Table>("MODEL_LOG")
            .expect("the model log")
            .sequence_values::<mlua::Table>()
            .map(|entry| entry.expect("a model log entry"))
            .filter(|entry| entry.get::<String>("op").unwrap() == "entity_candidate")
            // a call the scripted provider raised for has no answer: it is not listed
            .filter_map(|entry| {
                let answer = entry.get::<Option<mlua::Table>>("answer").unwrap()?;
                Some((
                    entry.get::<i64>("scene").unwrap(),
                    entry.get::<String>("key").unwrap(),
                    answer.get::<String>("result").unwrap(),
                    answer.get::<Option<String>>("reason").unwrap().unwrap_or_default(),
                ))
            })
            .collect(),
        candidate_log: candidates
            .sequence_values::<mlua::Table>()
            .map(|entry| {
                let entry = entry.expect("a candidate log entry");
                (
                    entry.get::<i64>("scene").unwrap(),
                    entry.get::<String>("result").unwrap(),
                    entry.get::<String>("key").unwrap(),
                )
            })
            .collect(),
    }
}

fn rows(driven: &Driven) -> Vec<Value> {
    driven
        .rows
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("{line}: {e}")))
        .collect()
}

fn row_of(driven: &Driven, event: &str) -> Value {
    rows(driven)
        .into_iter()
        .find(|row| row["event"] == event)
        .unwrap_or_else(|| panic!("no {event} row: {}", driven.rows))
}

fn authority(driven: &Driven) -> Value {
    row_of(driven, "authority")
}

fn error_of(driven: Driven) -> String {
    driven.result.expect_err("the run must fail")
}

const TERMINAL: &str = "\"kind=terminal w=5 type=gun code=ground steps=40 pos=1.5,100.5,3 velocity=3 lifetime=4\"";
const TERMINAL_HIT: &str = "\"kind=terminal w=5 type=gun code=target steps=1 pos=100,50,200 velocity=0 lifetime=0.1\"";

/// a healthy scenario: the weapon ends at the first frame after the launch, on the ground; when a hit reaches it, on the target
fn healthy(extra: &str) -> String {
    format!("after = {{ [1] = {{ {TERMINAL} }} }}, after_hit = {{ [1] = {{ {TERMINAL_HIT} }} }}{extra}")
}

/// The launch report of the real C emitter (eech_wpn_fixture.c wpn-case-launch), with single fields replaced.
fn report(changes: &str) -> String {
    let mut fields: Vec<(String, String)> = [
        ("launcher", "helicopter"),
        ("weapon", "7"),
        ("target_link_before", "0"),
        ("target_link", "1"),
        ("count_before", "3"),
        ("count_after", "2"),
        ("births", "1"),
        ("creates", "1"),
        ("restored", "1"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    for change in changes.split_whitespace() {
        let (key, value) = change.split_once('=').expect("key=value");
        fields.iter_mut().find(|(k, _)| k == key).expect("a known field").1 = value.to_string();
    }
    let body: Vec<String> = fields.iter().map(|(k, v)| format!("{k}={v}")).collect();
    format!("wpn-case-launch {}", body.join(" "))
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the healthy control: reached, well formed, joined with the fixture's counters

#[test]
fn a_healthy_control_sees_real_entity_questions_checks_their_shape_and_joins_the_fixture_counters() {
    for (extra, stationary, moving) in [("", 1, 0), (", moving = true", 0, 1), (", questions = 3", 3, 0)] {
        let driven = drive(&healthy(extra), &[]);
        let outcome = driven.result.clone();
        assert!(outcome.is_ok(), "{extra}: {outcome:?}\n{}", driven.rows);
        let row = authority(&driven);
        assert_eq!(row["completed"], true);
        assert_eq!(
            (row["stationary"].clone(), row["moving"].clone()),
            (Value::from(stationary), Value::from(moving)),
            "{extra}"
        );
        assert_eq!(
            row["entity_after"],
            stationary + moving,
            "{extra}: every question after the launch was counted, none before"
        );
        assert_eq!(row["entity_before"], 0);
        assert_eq!((row["request_faults"].clone(), row["reply_faults"].clone()), (Value::from(0), Value::from(0)));
        assert_eq!(row["real_miss"], stationary + moving);
        assert_eq!(
            row["real_mask_56"],
            stationary + moving,
            "the real reply is a miss that assigned only the normal"
        );
        assert_eq!((row["altered"].clone(), row["answered_hit"].clone()), (Value::from(0), Value::from(0)));
        assert_eq!(row["contact_any_tests_delta"], 1, "the fixture's own count of the weapon step's entity tests");
        assert_eq!(row["contact_any_hits_delta"], 0);
        assert_eq!(row["precision_high_delta"], 1);
        assert_eq!(row["attribution"], "global_to_the_war_no_per_weapon_identity");
        assert!(driven.delegation_broken.is_empty(), "{extra}: {:?}", driven.delegation_broken);
        assert_eq!(driven.closed, 1, "the evidence is closed once, on success");
    }
}

#[test]
fn a_real_hit_is_checked_against_the_weapon_steps_own_hit_count() {
    // the host's narrow phase reports a hit and the weapon step counted it: coherent, and the terminal is the on-target one
    let driven = drive(&healthy(", real = 'hit'"), &[]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    let row = authority(&driven);
    assert_eq!(
        (row["real_hit"].clone(), row["real_mask_63"].clone(), row["contact_any_hits_delta"].clone()),
        (Value::from(1), Value::from(1), Value::from(1))
    );
    // the weapon step counted a hit the host's narrow phase never reported: the answer was used elsewhere or the count is wrong
    let error = error_of(drive(&healthy(", hits_despite_miss = true"), &[]));
    assert!(error.contains("counted 1 entity hits but the host's narrow phase reported only 0"), "{error}");
}

#[test]
fn the_request_is_the_corrected_shape_and_a_malformed_request_or_real_reply_is_a_failure() {
    // positive first: the logged question is the segment and the pose, 18 patterns (stationary) or 22 (moving), and carries no outputs
    let driven = drive(&healthy(""), &[]);
    assert_eq!(driven.entity_log.len(), 1);
    assert_eq!(driven.entity_log[0].0.split(',').count(), 18);
    let driven = drive(&healthy(", moving = true"), &[]);
    assert_eq!(driven.entity_log[0].0.split(',').count(), 22);
    // single faults of the request
    for (fault, fragment) in [
        ("count", "the pose has 17 floats"),
        ("token", "not comma separated"),
        ("extra", "carries only a scene and the pose text"),
    ] {
        let error = error_of(drive(&healthy(&format!(", bad_request = '{fault}'")), &[]));
        assert!(error.contains("request_faults=1") && error.contains(fragment), "{fault}: {error}");
    }
    // single faults of the host's own reply. The driver's own contract check, with a scripted campaign that would accept the reply:
    let faults = [
        ("unassigned_word", "word 1 is not assigned by mask 56 but is not zero"),
        ("hit_mask", "a hit with mask 56"),
        ("partial", "assigns a vector in part"),
    ];
    for (real, fragment) in faults {
        let error = error_of(drive(&healthy(&format!(", real = '{real}', lenient_engine = true")), &[]));
        assert!(error.contains("reply_faults=1") && error.contains(fragment), "{real}: {error}");
    }
    // an altered mode never forms its answer from a malformed real reply: the host's own answer is returned unaltered (no witness failure crosses the engine's callback boundary, where it would
    // carry the callback marker the refusal modes credit) and the fault fails the verdict
    let driven = drive(&healthy(", real = 'hit_mask', lenient_engine = true"), &[("mode", "forced_hit")]);
    let rows_text = driven.rows.clone();
    let error = error_of(driven);
    assert!(error.contains("reply_faults=1"), "{error}");
    assert!(rows_text.contains("\"altered\":0"), "{rows_text}");
    // and the campaign's own rules stop the engine on the same replies: the control cannot complete, and the stop is the engine's, never a pass
    for (real, marker) in [
        ("unassigned_word", "entity_collision.unwritten"),
        ("hit_mask", "entity_collision.write_mask"),
        ("partial", "entity_collision.write_mask"),
    ] {
        let error = error_of(drive(&healthy(&format!(", real = '{real}'")), &[]));
        assert!(
            error.contains(&format!("mode control failed in the engine at [model-provider:{marker}]")),
            "{real}: {error}"
        );
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// reached, unreached, API missing

#[test]
fn a_run_whose_actual_entity_caller_or_narrow_phase_was_not_reached_is_unreached_never_authority() {
    // the weapon step's entity lookup was made but no candidate passed the broad phase: the host was asked nothing
    let error = error_of(drive(&healthy(", no_entity_questions = true"), &[]));
    assert!(
        error.contains("UNREACHED") && error.contains("no candidate reached the narrow phase"),
        "{error}"
    );
    // the weapon step never made the lookup: the actual caller was not reached
    let error = error_of(drive(&healthy(", no_entity_tests = true"), &[]));
    assert!(error.contains("UNREACHED") && error.contains("contact_any_tests did not rise"), "{error}");
    // the host ignores the viewed answer: the step never took the high-precision path
    let error = error_of(drive(&healthy(", view_ignored = true"), &[]));
    assert!(error.contains("UNREACHED") && error.contains("precision_high did not rise"), "{error}");
    // the campaign sent no VIEW_WEAPON answer at all
    let error = error_of(drive(&healthy(", no_view_answer = true"), &[]));
    assert!(error.contains("UNREACHED") && error.contains("never reached a weapon step"), "{error}");
    // a VIEW_WEAPON answer that does not carry twelve result words cannot be applied exactly: a witness failure, not a guess
    let error = error_of(drive(&healthy(", short_view = true"), &[]));
    assert!(error.contains("twelve result words"), "{error}");
    // a view the campaign already answered as set is counted, not hidden
    let driven = drive(&healthy(", view_preset = 1"), &[]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    assert_eq!(authority(&driven)["view_already_set"], 1);
    // the evidence still records what was seen when the run is UNREACHED
    let driven = drive(&healthy(", no_entity_questions = true"), &[]);
    assert_eq!(authority(&driven)["entity_after"], 0);
}

#[test]
fn the_armed_weapon_step_is_given_the_configured_view_and_everything_else_reaches_the_host_unchanged() {
    let driven = drive(&healthy(""), &[]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    let row = authority(&driven);
    assert_eq!(row["view_input"], "configured_view_weapon_true_on_armed_weapon_steps");
    assert_eq!(
        (
            row["view_steps_seen"].clone(),
            row["view_answers_seen"].clone(),
            row["view_applied"].clone(),
            row["view_already_set"].clone()
        ),
        (Value::from(2), Value::from(1), Value::from(1), Value::from(0))
    );
    assert!(!driven.caller_mutated, "the caller's request table must not change");
    for (mode, armed, paths, zero) in &driven.aim_log {
        // only an ARMED weapon_step WITH the lookup answer differs, and only in the sixth result word of the VIEW_WEAPON answer (f[7] in Lua)
        let expected = if *armed && mode == "weapon_step" && zero != "-" {
            "answers[2].f[7]"
        } else {
            ""
        };
        assert_eq!(paths, expected, "{mode} armed={armed}: {paths}");
    }
    let armed_full: Vec<_> = driven
        .aim_log
        .iter()
        .filter(|(mode, armed, paths, _)| *armed && mode == "weapon_step" && !paths.is_empty())
        .collect();
    assert_eq!(armed_full.len(), 1, "{:?}", driven.aim_log);
    assert_eq!(
        armed_full[0].3, "-0",
        "a signed zero in an unrelated result word reached the host as a signed zero"
    );
}

#[test]
fn every_other_operation_and_every_question_before_the_launch_is_the_hosts_own_answer_unchanged_in_every_mode() {
    for mode in ["control", "forced_hit", "forced_miss"] {
        let driven = drive(&healthy(""), &[("mode", mode)]);
        let outcome = driven.result.clone();
        assert!(outcome.is_ok(), "{mode}: {outcome:?}\n{}", driven.rows);
        assert!(driven.delegation_broken.is_empty(), "{mode}: {:?}", driven.delegation_broken);
        let row = authority(&driven);
        assert!(
            row["other_ops_before"].as_i64().unwrap() >= 2 && row["other_ops_after"].as_i64().unwrap() >= 2,
            "{mode}: other operations were asked before and after the launch"
        );
    }
    // the check is real: a wrapper that changed an operation would be recorded; here, the scripted engine proves it can see a difference by comparing identity
    let driven = drive(&healthy(", no_other_ops = true"), &[]);
    assert!(driven.result.is_ok());
    assert_eq!(authority(&driven)["other_ops_after"], 0);
}

#[test]
fn the_original_profile_is_a_capability_red_only_when_the_actual_entity_caller_ran_and_otherwise_unreached() {
    let original = [("profile", "original")];
    // the scripted original makes entity tests (the weapon fixture's counter rises) but never asks the host, and the fixture API is present: the expected RED
    let error = error_of(drive(&healthy(", fixture_api = true, original_tests = true"), &original));
    assert!(
        error.contains("CAPABILITY RED") && error.contains("contact_any_tests +1") && error.contains("fixture API present"),
        "{error}"
    );
    // the same, with a hit among the lookups: the row says how many lookups hit
    let error = error_of(drive(&healthy(", fixture_api = true, original_tests = true, original_hits = true"), &original));
    assert!(error.contains("CAPABILITY RED") && error.contains("contact_any_hits +1"), "{error}");
    // a module without the fixture API is not the original profile: setup failure, not a red
    let error = error_of(drive(&healthy(", original_tests = true"), &original));
    assert!(error.contains("UNREACHED") && error.contains("collision-fixture API"), "{error}");
    // with the API but no entity test (the counter did not rise) it is UNREACHED too: the call the red is about never happened
    let error = error_of(drive(&healthy(", fixture_api = true, no_entity_tests = true"), &original));
    assert!(error.contains("UNREACHED") && error.contains("contact_any_tests did not rise"), "{error}");
    // the Original profile gets the identical configured input, and it is recorded
    let driven = drive(&healthy(", fixture_api = true, original_tests = true"), &original);
    let rows_text = driven.rows.clone();
    assert!(error_of(driven).contains("CAPABILITY RED"));
    assert!(
        rows_text.contains("\"view_applied\":1") && rows_text.contains("\"precision_high_delta\":1") && rows_text.contains("\"fixture_api\":true"),
        "{rows_text}"
    );
    // a module that is NOT the original but is asked in the original profile (the host was asked a question) is refused
    let error = error_of(drive(&healthy(", fixture_api = true"), &original));
    assert!(error.contains("the module is not the original"), "{error}");
    // the original profile only runs the control; the hosted profile refuses a module that has the original's fixture API
    assert!(error_of(drive("", &[("profile", "original"), ("mode", "forced_hit")])).contains("only mode=control"));
    let error = error_of(drive(&healthy(", fixture_api = true"), &[]));
    assert!(error.contains("keeps the original narrow phase"), "{error}");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the altered modes

#[test]
fn a_forced_hit_is_a_valid_complete_answer_at_the_requests_old_end_and_must_reach_the_weapon_steps_hit_count() {
    let control = drive(&healthy(""), &[]);
    let forced = drive(&healthy(""), &[("mode", "forced_hit")]);
    let outcome = forced.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", forced.rows);
    let row = authority(&forced);
    assert_eq!(
        (row["altered"].clone(), row["answered_hit"].clone(), row["answered_miss"].clone()),
        (Value::from(1), Value::from(1), Value::from(0))
    );
    assert_eq!(row["contact_any_hits_delta"], 1, "the forced hit reached the weapon step's own hit count");
    // the answer is complete write information: mask 63, the request's OLD end as the point, an up normal; the real host answer is still asked (its name is the echo)
    let (key, result, mask, p) = &forced.entity_log[0];
    let old: Vec<&str> = key.split(',').take(3).collect();
    assert_eq!((result.as_str(), *mask), ("hit", 63));
    assert_eq!(p, &format!("{},{},0x3f800000,{}", old.join(","), "0x00000000", "0x00000000"));
    // the PHYSICAL signature of the cohort changes against the control: its terminal is the on-target one, and it is compared without the weapon's ordinal
    let (control_signature, forced_signature) = (authority(&control)["cohort_signature"].clone(), row["cohort_signature"].clone());
    assert_ne!(control_signature, forced_signature, "the forced hit changed the cohort's physical terminal");
    assert!(forced_signature.as_str().unwrap().starts_with("gun|target|"), "{forced_signature}");
    assert!(!forced_signature.as_str().unwrap().contains("w=5"), "the signature carries no ordinal");
    // a forced hit that the actual caller never counted is a failure: the answer was not used
    let error = error_of(drive(&healthy(", hit_ignored = true"), &[("mode", "forced_hit")]));
    assert!(error.contains("never reached the weapon step's entity hit count"), "{error}");
    // a mode that altered nothing (no question) is UNREACHED, not a pass
    let error = error_of(drive(&healthy(", no_entity_questions = true"), &[("mode", "forced_hit")]));
    assert!(error.contains("UNREACHED"), "{error}");
}

#[test]
fn a_forced_miss_is_an_accepted_partial_write_miss_and_no_hit_may_reach_the_weapon_step() {
    let driven = drive(&healthy(""), &[("mode", "forced_miss")]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    let row = authority(&driven);
    assert_eq!(
        (row["altered"].clone(), row["answered_miss"].clone(), row["answered_hit"].clone()),
        (Value::from(1), Value::from(1), Value::from(0))
    );
    assert_eq!(row["contact_any_hits_delta"], 0);
    // the answer assigns the normal only: mask 56, the point's three words are zero
    let (_, result, mask, p) = &driven.entity_log[0];
    assert_eq!((result.as_str(), *mask), ("miss", 56));
    assert_eq!(p, "0x00000000,0x00000000,0x00000000,0x00000000,0x3f800000,0x00000000");
    // honest about what it shows: valid MISS coverage, not a measured physical change
    assert_eq!(
        row["miss_coverage"],
        "valid_accepted_miss_not_a_measured_physical_change_unless_the_control_had_hits"
    );
    // a hit that still reaches the weapon step after every question was answered as a miss is a failure
    let error = error_of(drive(&healthy(", hits_despite_miss = true"), &[("mode", "forced_miss")]));
    assert!(error.contains("still counted 1 entity hits"), "{error}");
}

#[test]
fn a_refusal_counts_only_at_the_anchored_engine_stop_with_the_exact_marker_and_nothing_is_asked_afterwards() {
    for (mode, marker) in [
        ("withheld_entity", "entity_collision.callback"),
        ("bad_write_mask", "entity_collision.write_mask"),
        ("bad_answer", "entity_collision.answer"),
    ] {
        let driven = drive(&healthy(""), &[("mode", mode)]);
        let outcome = driven.result.clone();
        assert!(outcome.is_ok(), "{mode}: {outcome:?}\n{}", driven.rows);
        let row = authority(&driven);
        assert_eq!(row["completed"], false, "{mode}");
        assert_eq!(row["failure_marker"], marker, "{mode}");
        assert_eq!(row["altered"], 1, "{mode}");
        assert_eq!(row["case_weapons_pending"], -1, "{mode}: the cohort is not counted after the stop");
        assert_eq!(row["contact_any_tests_delta"], -1, "{mode}: no counter is read from a stopped engine");
        assert_eq!(row["contact_any_hits_delta"], -1, "{mode}: unknown stays unknown");
    }
    // the same stop at the WRONG marker (an answer cause where a callback was intended) is the witness failing
    let error = error_of(drive(
        &healthy(", engine_stop = { after = 1, marker = 'entity_collision.answer' }"),
        &[("mode", "withheld_entity")],
    ));
    assert!(
        error.contains("stopped at [model-provider:entity_collision.answer], not at [model-provider:entity_collision.callback]"),
        "{error}"
    );
    // a Lua error that only mentions the marker, and an engine stop without the known wrapper, are not the engine's
    for scenario in [
        "engine_stop = { after = 1, marker = 'entity_collision.callback', text_only = true }",
        "engine_stop = { after = 1, marker = 'x', wrapper = 'eech: EECH fatal error: host model: [model-provider:entity_collision.callback] x' }",
        "engine_stop = { after = 1, marker = 'x', wrapper = 'runtime error: eech: EECH fatal error: host terrain: [terrain-provider:tree_begin.callback] x' }",
    ] {
        let error = error_of(drive(&healthy(&format!(", {scenario}")), &[("mode", "withheld_entity")]));
        assert!(error.contains("an unrecognised failure, not the engine's known fatal"), "{scenario}: {error}");
    }
    // a refusal mode whose flight never asks the host completes without the stop: that is a failure, not a pass
    let error = error_of(drive(&healthy(", no_entity_questions = true"), &[("mode", "withheld_entity")]));
    assert!(error.contains("the engine did not stop at entity_collision.callback"), "{error}");
    // a stop that happens although the provider altered nothing (an unaltered control question) is not credited to the mode
    let error = error_of(drive(
        &healthy(", engine_stop = { after = 1, marker = 'entity_collision.callback' }"),
        &[("mode", "withheld_entity")],
    ));
    assert!(error.contains("without the provider having altered"), "{error}");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the case: selection, cohort, launch report, terminals

fn selected(driven: &Driven) -> Value {
    row_of(driven, "case_selected")
}

#[test]
fn the_case_is_the_nearest_non_person_enemy_of_a_launcher_with_an_unguided_ballistic_weapon() {
    // the default roster: blue launcher at the origin, red target at 100, 0, 100
    let driven = drive(&healthy(""), &[]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    let row = selected(&driven);
    assert_eq!(
        (row["launcher"].clone(), row["target"].clone(), row["weapon"].clone()),
        (Value::from("eech-world:1"), Value::from("eech-world:2"), Value::from(7))
    );
    assert_eq!(row["selection"], "nearest_non_person_enemy_of_a_launcher_with_an_unguided_ballistic_weapon");
    // a nearer PERSON is not a candidate (no collision flag), a nearer FRIEND is not an enemy; the nearer vehicle is chosen
    let scenario = healthy(
        ", mobiles = { { kind = 5, side = 'blue' }, { kind = 1, side = 'red' }, { kind = 1, side = 'red' }, { kind = 3, side = 'red' }, { kind = 1, side = 'blue' } }, positions = { '0,0,0', '100,0,100', '40,0,0', '5,0,0', '2,0,0' }",
    );
    let driven = drive(&scenario, &[]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    assert_eq!(selected(&driven)["target"], "eech-world:3");
    // the smallest distance wins ACROSS launchers: the second launcher is nearer to its enemy than the first is to its own
    let scenario = healthy(
        ", mobiles = { { kind = 5, side = 'blue' }, { kind = 1, side = 'red' }, { kind = 1, side = 'blue' } }, positions = { '0,0,0', '100,0,0', '98,0,0' }, armed = { [1] = true, [3] = true }",
    );
    let driven = drive(&scenario, &[]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    let row = selected(&driven);
    assert_eq!(
        (row["launcher"].clone(), row["target"].clone()),
        (Value::from("eech-world:3"), Value::from("eech-world:2"))
    );
    assert_eq!(row["distance"], 2.0);
    // the nearest pair's launcher has no weapon in stock: the next nearest pair is taken
    let scenario = healthy(
        ", mobiles = { { kind = 5, side = 'blue' }, { kind = 1, side = 'red' }, { kind = 1, side = 'blue' } }, positions = { '0,0,0', '100,0,0', '98,0,0' }, armed = { [1] = true }",
    );
    let driven = drive(&scenario, &[]);
    assert_eq!(selected(&driven)["launcher"], "eech-world:1");
    // the pair that was chosen is the one the launch was made with
    assert_eq!(driven.launch, Some(("eech-world:1".to_string(), "eech-world:2".to_string(), 7)));
    // no enemy that is a candidate type at all: a setup failure, not a guess
    let scenario = healthy(", mobiles = { { kind = 5, side = 'blue' }, { kind = 3, side = 'red' } }");
    let error = error_of(drive(&scenario, &[]));
    assert!(error.contains("SETUP FAILURE: no launcher"), "{error}");
}

#[test]
fn an_incomplete_empty_or_inconsistent_cohort_is_a_failure_and_never_a_pass() {
    let two = "births = { 'kind=birth w=5 type=gun', 'kind=birth w=6 type=gun' }";
    let error = error_of(drive(&format!("{two}, {}", healthy("")), &[]));
    assert!(error.contains("INCOMPLETE") && error.contains("1 of 2"), "{error}");
    let error = error_of(drive(&format!("births = {{}}, launch_report = '{}'", report("births=0")), &[]));
    assert!(error.contains("the controlled cohort is empty"), "{error}");
    let error = error_of(drive(&format!("launch_report = '{}', {}", report("births=2"), healthy("")), &[]));
    assert!(error.contains("births=2 but the launch logged 1"), "{error}");
    let error = error_of(drive(&format!("launch_report = '{}', {}", report("restored=0"), healthy("")), &[]));
    assert!(error.contains("restore the launcher's temporary target link"), "{error}");
    let error = error_of(drive(
        &format!(
            "launch_report = 'something else {}', {}",
            report("").replace("wpn-case-launch ", ""),
            healthy("")
        ),
        &[],
    ));
    assert!(error.contains("not a wpn-case-launch line"), "{error}");
    // a duplicate terminal and an incomplete one
    let error = error_of(drive(
        &format!("after = {{ [1] = {{ {TERMINAL}, \"kind=terminal w=5 type=gun code=target steps=41 pos=2,2,2 velocity=3 lifetime=4\" }} }}"),
        &[],
    ));
    assert!(error.contains("ended twice"), "{error}");
    let error = error_of(drive(
        "after = { [1] = { 'kind=terminal w=5 type=gun steps=40 pos=1,1,1 velocity=3 lifetime=4' } }",
        &[],
    ));
    assert!(error.contains("is incomplete") && error.contains("field code"), "{error}");
    // natural activity of other weapons is declared foreign and is not the case's
    let scenario = format!(
        "after = {{ [1] = {{ 'kind=birth w=9 type=later', 'kind=terminal w=9 type=later code=target steps=3 pos=7,8,9 velocity=1 lifetime=1' }}, [2] = {{ {TERMINAL} }} }}"
    );
    let driven = drive(&scenario, &[]);
    let outcome = driven.result.clone();
    assert!(outcome.is_ok(), "{outcome:?}\n{}", driven.rows);
    let row = authority(&driven);
    assert_eq!(
        (row["foreign_terminals"].clone(), row["later_births"].clone(), row["cohort_size"].clone()),
        (Value::from(1), Value::from(1), Value::from(1))
    );
    assert_eq!(row["case_weapons_ended"], 1, "only the case weapon's terminal is the case's");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the clock, the output, the arguments and the shared words

#[test]
fn setup_output_and_arguments_fail_closed_and_the_first_error_is_never_hidden() {
    // positive first: the healthy run opens the evidence through the checked exclusive-create capability and closes it once
    let driven = drive(&healthy(""), &[]);
    assert!(driven.result.is_ok() && driven.created && driven.closed == 1);
    assert!(
        SCRIPT.contains("host.create_exclusive") && !SCRIPT.contains("host.output_create_new ("),
        "the witness must use host.create_exclusive"
    );
    // a host with no model provider or no terrain factory fails AFTER the evidence was opened: it is closed once and the failure is named
    for (scenario, fragment) in [("no_model = true", "no host.model"), ("no_terrain = true", "no host.terrain_provider")] {
        let driven = drive(scenario, &[]);
        let (created, closed) = (driven.created, driven.closed);
        let error = error_of(driven);
        assert!(error.contains(fragment), "{scenario}: {error}");
        assert!(created && closed == 1, "{scenario}: opened {created}, closed {closed}");
        let driven = drive(&format!("{scenario}, io_fail = {{ op = 'close' }}"), &[]);
        let error = error_of(driven);
        assert!(error.contains(fragment) && !error.contains("closing"), "{scenario}: {error}");
    }
    let stalled = drive("clock_step = 0", &[]);
    let closed = stalled.closed;
    assert!(error_of(stalled).contains("entity-authority: STALLED CLOCK"));
    assert_eq!(closed, 1, "the evidence is closed on the error path too");
    for (op, at, fragment) in [
        ("write", 1, "OUTPUT FAILURE: writing"),
        ("write", 4, "OUTPUT FAILURE: writing"),
        ("flush", 2, "OUTPUT FAILURE: flushing"),
        ("close", 0, "OUTPUT FAILURE: closing"),
    ] {
        let driven = drive(&format!("io_fail = {{ op = '{op}', at = {at} }}, {}", healthy("")), &[]);
        let error = error_of(driven);
        assert!(error.contains(&format!("entity-authority: {fragment}")), "{op} {at}: {error}");
    }
    let existing = drive("create_fails = true", &[]);
    assert!(!existing.created && existing.rows.is_empty());
    assert!(error_of(existing).contains("nothing was run"));
    for (key, value) in [
        ("warmup", "nan"),
        ("settle", "inf"),
        ("frame_ms", "0"),
        ("warmup", "100000"),
        ("profile", "elsewhere"),
        ("mode", "nothing"),
        ("mode", "withheld_tree"),
    ] {
        let refused = drive("", &[(key, value)]);
        assert!(!refused.created, "{key}={value}");
        assert!(error_of(refused).contains("entity-authority:"), "{key}={value}");
    }
}

#[test]
fn the_words_helper_is_the_same_text_as_the_tree_witnesss() {
    // the helper that gives the exact float words (both zeros, subnormals, boundaries) was proven under an enabled divide-by-zero trap for the tree witness; this one must be the same code
    let block = |text: &str| -> Vec<String> {
        let begin = text.find("-- BEGIN words").expect("the words marker");
        let end = text.find("-- END words").expect("the words end marker");
        text[begin..end]
            .lines()
            .filter(|line| !line.trim_start().starts_with("--"))
            .map(|line| line.to_string())
            .collect()
    };
    let (entity, tree) = (block(SCRIPT), block(TREE_SCRIPT));
    assert!(entity.iter().any(|line| line.contains("local function word_of")), "the block holds the helper");
    assert_eq!(entity, tree);
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the ENTITY_CANDIDATE path: the real candidate dialogue of the scripted scan (44 facts, 7 states), the natural control, the paired controls and the refusals

/// the states a staged dialogue passes through before its test: the fixed facts, the placement and the terrain normal (a non-mobile candidate that starts with scene 0)
const STAGED: &str = ", fixed_candidate = true, candidate_sequence = { 'need_fixed', 'need_placement', 'need_normal', 'test' }";

fn candidate_row(extra: &str, mode: &str) -> (Driven, Value) {
    let driven = drive(&healthy(extra), &[("mode", mode)]);
    let row = authority(&driven);
    (driven, row)
}

#[test]
fn the_natural_control_counts_the_actual_candidate_questions_and_classifies_the_real_answers_with_nothing_altered() {
    // one mobile candidate with its scene, told to test it
    let (driven, row) = candidate_row("", "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(
        (row["candidate_before"].clone(), row["candidate_after"].clone()),
        (Value::from(0), Value::from(1))
    );
    assert_eq!(
        (row["candidate_test"].clone(), row["candidate_mobile"].clone(), row["candidate_scene0"].clone()),
        (Value::from(1), Value::from(1), Value::from(0))
    );
    assert_eq!(
        (row["entity_after"].clone(), row["candidate_altered"].clone(), row["altered"].clone()),
        (Value::from(1), Value::from(0), Value::from(0))
    );
    assert_eq!(row["secondary_input"], "", "the natural control holds no configured secondary input");
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);
    // a staged dialogue of a non-mobile candidate: scene 0 first, a scene once its fixed facts were read, every state counted
    let (driven, row) = candidate_row(STAGED, "control");
    // no MOBILE candidate was offered, so the observation of the mobile collaborator contract was never reached: the control is UNREACHED for that fact and says so, while every candidate state is still counted
    let error = driven.result.clone().expect_err("the run must fail");
    assert!(error.contains("UNREACHED") && error.contains("asked no entity_pose question"), "{error}");
    assert_eq!(row["pose_after"], 0);
    assert_eq!(row["candidate_after"], 4);
    for (field, expected) in [
        ("candidate_need_fixed", 1),
        ("candidate_need_placement", 1),
        ("candidate_need_normal", 1),
        ("candidate_test", 1),
        ("candidate_scene0", 1),
        ("candidate_mobile", 0),
        ("entity_after", 1),
    ] {
        assert_eq!(row[field], expected, "{field}");
    }
    assert_eq!(
        driven
            .candidate_log
            .iter()
            .map(|(scene, result, _)| (*scene, result.as_str()))
            .collect::<Vec<_>>(),
        [(0, "need_fixed"), (9, "need_placement"), (9, "need_normal"), (9, "test")]
    );
    assert!(
        driven.candidate_log.iter().all(|(_, _, facts)| facts.split(',').count() == 44),
        "every question carries exactly 44 patterns"
    );
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);
    // three scan candidates in one flight
    let (driven, row) = candidate_row(", questions = 3", "control");
    assert!(driven.result.is_ok(), "{:?}", driven.result);
    assert_eq!((row["candidate_after"].clone(), row["entity_after"].clone()), (Value::from(3), Value::from(3)));
}

#[test]
fn zero_candidate_questions_is_unreached_even_when_the_narrow_phase_was_asked_and_a_malformed_request_or_real_reply_is_a_failure() {
    // the narrow phase was asked but no candidate question ever reached the host: the real scan was not reached, so it is not authority
    let error = error_of(drive(&healthy(", no_candidate_questions = true"), &[]));
    assert!(error.contains("UNREACHED") && error.contains("asked no entity_candidate question"), "{error}");
    // the scan offered a candidate and the host's real decision was a skip: the natural control then reached no narrow phase, which is UNREACHED
    let driven = drive(&healthy(", candidate_sequence = { 'skip' }"), &[]);
    let row = authority(&driven);
    assert_eq!(
        (row["candidate_after"].clone(), row["candidate_skip"].clone(), row["entity_after"].clone()),
        (Value::from(1), Value::from(1), Value::from(0))
    );
    assert!(error_of(driven).contains("no candidate reached the narrow phase"));
    // a request that is not 44 patterns is counted and fails the verdict, never credited
    let driven = drive(&healthy(", bad_candidate_request = 'count'"), &[]);
    let row = authority(&driven);
    assert_eq!(row["request_faults"], 1);
    assert!(row["first_fault"].as_str().unwrap().starts_with("candidate request:"), "{row}");
    // real replies that break the contract are counted at the question and stop the scripted engine at the candidate's own marker
    for (bad, marker) in [
        ("state", "entity_candidate.answer"),
        ("name0", "entity_candidate.identity"),
        ("unnamed", "entity_candidate.identity"),
    ] {
        // the unnamed reply is wrong only for a candidate that has a scene (the mobile one); the others are wrong at scene 0
        let scene0 = if bad == "unnamed" { "" } else { ", fixed_candidate = true" };
        let driven = drive(
            &healthy(&format!(", bad_real_candidate = '{bad}', candidate_sequence = {{ 'skip' }}{scene0}")),
            &[],
        );
        let row = authority(&driven);
        assert_eq!(row["reply_faults"], 1, "{bad}");
        assert!(row["first_fault"].as_str().unwrap().starts_with("candidate reply:"), "{bad}: {row}");
        let error = error_of(driven);
        assert!(error.contains(marker), "{bad}: {error}");
    }
}

#[test]
fn the_paired_control_reaches_the_actual_narrow_phase_with_the_configured_hit_and_the_cohort_physically_hits() {
    let natural = drive(&healthy(""), &[]);
    let (paired, row) = candidate_row("", "candidate_real");
    assert!(paired.result.is_ok(), "{:?}\n{}", paired.result, paired.rows);
    assert_eq!(row["completed"], true);
    // the candidate answers are the host's real ones; only the configured narrow-phase hit is held, and it is declared
    assert_eq!(
        (row["candidate_altered"].clone(), row["candidate_test"].clone(), row["candidate_after"].clone()),
        (Value::from(0), Value::from(1), Value::from(1))
    );
    assert_eq!(
        (row["entity_after"].clone(), row["answered_hit"].clone(), row["altered"].clone()),
        (Value::from(1), Value::from(1), Value::from(1))
    );
    assert_eq!(row["contact_any_hits_delta"], 1, "the actual caller used the configured hit");
    assert!(row["secondary_input"].as_str().unwrap().contains("configured_not_natural"), "{row}");
    assert!(row["cohort_signature"].as_str().unwrap().contains("target"), "the cohort physically hit");
    assert_ne!(
        row["cohort_signature"],
        authority(&natural)["cohort_signature"],
        "and that is not the natural healthy signature"
    );
    // a hit the actual caller did not use is not authority
    let error = error_of(drive(&healthy(", hit_ignored = true"), &[("mode", "candidate_real")]));
    assert!(error.contains("never reached the weapon step's entity hit count"), "{error}");
    // real decisions that never required a narrow phase leave the paired variant with nothing to prevent
    let error = error_of(drive(&healthy(", candidate_sequence = { 'skip' }"), &[("mode", "candidate_real")]));
    assert!(
        error.contains("UNREACHED") && error.contains("no candidate reached the narrow phase"),
        "{error}"
    );
}

#[test]
fn the_skip_variant_changes_the_cohort_by_preventing_every_narrow_phase_question_and_hit_with_the_same_configured_secondary_input() {
    let (paired, paired_row) = candidate_row("", "candidate_real");
    assert!(paired.result.is_ok());
    for extra in ["", ", questions = 3"] {
        let (skipped, row) = candidate_row(extra, "candidate_skip");
        assert!(skipped.result.is_ok(), "{extra}: {:?}\n{}", skipped.result, skipped.rows);
        let asked = if extra.is_empty() { 1 } else { 3 };
        assert_eq!(
            (
                row["candidate_after"].clone(),
                row["candidate_answered_skip"].clone(),
                row["candidate_altered"].clone()
            ),
            (Value::from(asked), Value::from(asked), Value::from(asked)),
            "{extra}"
        );
        assert_eq!(
            (row["entity_after"].clone(), row["altered"].clone(), row["contact_any_hits_delta"].clone()),
            (Value::from(0), Value::from(0), Value::from(0)),
            "{extra}"
        );
        assert_eq!(
            row["secondary_input"], paired_row["secondary_input"],
            "the same configured secondary input is declared in both paired modes"
        );
        assert_eq!(row["completed"], true);
        // the physical consequence: a cohort that no longer ends on the target
        assert_ne!(row["cohort_signature"], paired_row["cohort_signature"], "{extra}");
        assert!(row["cohort_signature"].as_str().unwrap().contains("ground"), "{extra}");
        assert!(skipped.delegation_broken.is_empty(), "{:?}", skipped.delegation_broken);
    }
    // a campaign that asks the narrow phase although the host skipped the candidate is a failure, not a pass
    let error = error_of(drive(&healthy(", ignore_skip = true"), &[("mode", "candidate_skip")]));
    assert!(
        error.contains("still asked 1 entity_collision questions after every candidate was skipped"),
        "{error}"
    );
    // no candidate question at all is UNREACHED for the variant too
    let error = error_of(drive(&healthy(", no_entity_questions = true"), &[("mode", "candidate_skip")]));
    assert!(error.contains("UNREACHED") && error.contains("asked no entity_candidate question"), "{error}");
}

#[test]
fn a_withheld_or_malformed_candidate_answer_counts_only_at_the_anchored_engine_stop_with_its_exact_candidate_marker_and_nothing_is_asked_afterwards() {
    const NO_SCENE: &str = ", fixed_candidate = true, candidate_sequence = { 'need_fixed', 'test' }";
    for (mode, marker, extra) in [
        ("withheld_candidate", "entity_candidate.callback", ""),
        ("bad_candidate_state", "entity_candidate.state", ""),
        ("bad_candidate_identity", "entity_candidate.identity", ""),
        // the same at a candidate that has no scene yet: the name check is an identity refusal there too
        ("bad_candidate_identity", "entity_candidate.identity", NO_SCENE),
        ("bad_candidate_state", "entity_candidate.state", NO_SCENE),
    ] {
        let driven = drive(&healthy(extra), &[("mode", mode)]);
        let outcome = driven.result.clone();
        assert!(outcome.is_ok(), "{mode} {extra}: {outcome:?}\n{}", driven.rows);
        let row = authority(&driven);
        assert_eq!(row["completed"], false, "{mode}");
        assert_eq!(row["failure_marker"], marker, "{mode}: the candidate's own marker, never the narrow phase's");
        assert_eq!(
            (row["candidate_altered"].clone(), row["altered"].clone()),
            (Value::from(1), Value::from(0)),
            "{mode}"
        );
        assert_eq!(row["entity_after"], 0, "{mode}: no narrow-phase question before or after the stop");
        assert_eq!(row["candidate_after"], 1, "{mode}: nothing is asked of the host after the fatal");
        assert_eq!(row["case_weapons_pending"], -1, "{mode}: the cohort is not counted after the stop");
        assert_eq!(row["contact_any_tests_delta"], -1, "{mode}: no counter is read from a stopped engine");
        assert_eq!(row["contact_any_hits_delta"], -1, "{mode}: unknown stays unknown");
    }
    // the old narrow-phase marker is not credited to a candidate mode, and a witness failure cannot pose as the host withholding an answer
    let error = error_of(drive(
        &healthy(", engine_stop = { after = 1, marker = 'entity_collision.callback' }"),
        &[("mode", "withheld_candidate")],
    ));
    assert!(
        error.contains("stopped at [model-provider:entity_collision.callback], not at [model-provider:entity_candidate.callback]"),
        "{error}"
    );
    // a stop at the candidate's own marker that the provider did not cause (nothing altered) is not credited either
    let error = error_of(drive(
        &healthy(", engine_stop = { after = 1, marker = 'entity_candidate.callback' }"),
        &[("mode", "withheld_candidate")],
    ));
    assert!(error.contains("without the provider having altered"), "{error}");
    for scenario in [
        "engine_stop = { after = 1, marker = 'entity_candidate.callback', text_only = true }",
        "engine_stop = { after = 1, marker = 'x', wrapper = 'eech: EECH fatal error: host model: [model-provider:entity_candidate.callback] x' }",
    ] {
        let error = error_of(drive(&healthy(&format!(", {scenario}")), &[("mode", "withheld_candidate")]));
        assert!(error.contains("an unrecognised failure, not the engine's known fatal"), "{scenario}: {error}");
    }
    // a flight that never asks the host completes without the stop: a failure, not a pass
    let error = error_of(drive(&healthy(", no_entity_questions = true"), &[("mode", "withheld_candidate")]));
    assert!(error.contains("the engine did not stop at entity_candidate.callback"), "{error}");
}

#[test]
fn the_original_profile_has_the_api_and_the_real_caller_lookups_but_no_candidate_question_and_the_expected_red_names_both_counts() {
    let original = [("profile", "original")];
    let driven = drive(&healthy(", fixture_api = true, original_tests = true"), &original);
    let row = authority(&driven);
    assert_eq!(
        (row["candidate_after"].clone(), row["entity_after"].clone(), row["fixture_api"].clone()),
        (Value::from(0), Value::from(0), Value::from(true))
    );
    let error = error_of(driven);
    assert!(
        error.contains("CAPABILITY RED") && error.contains("no entity_candidate question") && error.contains("fixture API present"),
        "{error}"
    );
    // an Original module that nevertheless asked the host a candidate question is not the original
    let error = error_of(drive(&healthy(", fixture_api = true"), &original));
    assert!(error.contains("entity_pose questions: the module is not the original"), "{error}");
    // the paired and candidate modes are hosted-profile modes only
    for mode in [
        "candidate_real",
        "candidate_skip",
        "withheld_candidate",
        "bad_candidate_state",
        "bad_candidate_identity",
    ] {
        assert!(
            error_of(drive("", &[("profile", "original"), ("mode", mode)])).contains("only mode=control"),
            "{mode}"
        );
    }
}

#[test]
fn the_candidate_modes_close_the_evidence_with_a_checked_close_and_a_failing_close_is_never_hidden() {
    let driven = drive(&healthy(", io_fail = { op = 'close' }"), &[("mode", "candidate_skip")]);
    assert_eq!(driven.closed, 1);
    assert!(error_of(driven).contains("OUTPUT FAILURE"), "the close failure is raised");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the ENTITY_SCAN path: the host drives the enumeration (16 facts, 5 states); the natural control, the paired controls and the refusals

fn scan_row(extra: &str, mode: &str) -> (Driven, Value) {
    let driven = drive(&healthy(extra), &[("mode", mode)]);
    let row = authority(&driven);
    (driven, row)
}

#[test]
fn the_natural_control_counts_the_actual_scan_questions_by_phase_and_requires_the_whole_chain_with_nothing_altered() {
    let (driven, row) = scan_row("", "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    // one walk: the start, the served entry, the resolved offer and the end of the list; one candidate dialogue and one narrow phase inside it
    assert_eq!((row["scan_before"].clone(), row["scan_after"].clone()), (Value::from(0), Value::from(4)));
    for (field, expected) in [
        ("scan_phase_begin", 1),
        ("scan_phase_entry", 1),
        ("scan_phase_resolved", 1),
        ("scan_phase_end", 1),
        ("scan_first", 1),
        ("scan_offer", 1),
        ("scan_next", 1),
        ("scan_done_none", 1),
        ("scan_done_hit", 0),
        ("scan_excluded", 0),
        ("scan_altered", 0),
        ("candidate_after", 1),
        ("entity_after", 1),
    ] {
        assert_eq!(row[field], expected, "{field}");
    }
    assert_eq!(row["secondary_input"], "", "the natural control holds no configured secondary input");
    assert_eq!(
        driven
            .scan_log
            .iter()
            .map(|(phase, result)| (phase.as_str(), result.as_str()))
            .collect::<Vec<_>>(),
        [
            ("0x00000000", "first"),
            ("0x3f800000", "offer"),
            ("0x40400000", "next"),
            ("0x40000000", "done_none")
        ]
    );
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);
    // three walks in one flight
    let (driven, row) = scan_row(", questions = 3", "control");
    assert!(driven.result.is_ok(), "{:?}", driven.result);
    assert_eq!(
        (row["scan_after"].clone(), row["candidate_after"].clone(), row["entity_after"].clone()),
        (Value::from(12), Value::from(3), Value::from(3))
    );
}

#[test]
fn zero_scan_questions_is_unreached_and_a_malformed_scan_request_or_real_reply_is_counted_and_never_credited() {
    // the narrow phase and the candidate dialogue were asked but the enumeration never reached the host: not authority
    let error = error_of(drive(&healthy(", no_scan_questions = true"), &[]));
    assert!(error.contains("UNREACHED") && error.contains("asked no entity_scan question"), "{error}");
    // the host's real enumeration offered nothing: positive scan questions, but nothing downstream, so the natural control is UNREACHED
    let driven = drive(&healthy(", scan_offers_nothing = true"), &[]);
    let row = authority(&driven);
    assert_eq!(
        (row["scan_after"].clone(), row["scan_done_none"].clone(), row["candidate_after"].clone()),
        (Value::from(1), Value::from(1), Value::from(0))
    );
    assert!(error_of(driven).contains("UNREACHED"));
    // a request that is not 16 patterns is counted
    let row = authority(&drive(&healthy(", bad_scan_request = 'count'"), &[]));
    assert!(row["request_faults"].as_i64().unwrap() >= 1, "{row}");
    assert!(row["first_fault"].as_str().unwrap().starts_with("scan request:"), "{row}");
    // real replies that break the grammar are counted at the question
    for bad in ["state", "p_on_next", "name", "first_one_pattern"] {
        let driven = drive(&healthy(&format!(", bad_real_scan = '{bad}'")), &[]);
        let row = authority(&driven);
        assert!(row["reply_faults"].as_i64().unwrap() >= 1, "{bad}: {row}");
        assert!(row["first_fault"].as_str().unwrap().starts_with("scan reply:"), "{bad}: {row}");
        assert!(driven.result.is_err(), "{bad}: a malformed real reply is never a pass");
    }
}

// the roster's events share the ENTITY_SCAN question with the scan itself (phases 4 to 6): the real traffic is the 16 facts, the key of the entry the scan stands at after them in ENTRY and RESOLVED,
// and the key the host names in FIRST, NEXT and an insertion. These controls run the real driver against the scripted collaborator speaking exactly that.

#[test]
fn real_protocol_traffic_with_roster_events_and_standing_keys_is_a_healthy_control_and_the_events_are_not_credited_as_query_scans() {
    let (driven, row) = scan_row("", "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    // one walk: the boot and one insertion are the roster's, the start, the served entry, the resolved offer and the end of the list the scan's
    assert_eq!((row["scan_before"].clone(), row["scan_after"].clone()), (Value::from(0), Value::from(4)));
    assert_eq!((row["roster_before"].clone(), row["roster_after"].clone()), (Value::from(0), Value::from(2)));
    for (field, expected) in [
        ("roster_insert", 1),
        ("roster_remove", 0),
        ("roster_reset", 1),
        ("roster_placed", 1),
        ("roster_removed", 0),
        ("roster_cleared", 1),
        ("scan_phase_begin", 1),
        ("scan_phase_entry", 1),
        ("scan_phase_resolved", 1),
        ("scan_phase_end", 1),
        ("scan_first", 1),
        ("scan_offer", 1),
        ("scan_next", 1),
        ("scan_done_none", 1),
        ("request_faults", 0),
        ("reply_faults", 0),
    ] {
        assert_eq!(row[field], expected, "{field}: {row}");
    }
    assert_eq!(
        driven.roster_log,
        [
            ("0x40c00000".to_string(), "cleared".to_string(), String::new(), String::new()),
            ("0x40800000".to_string(), "placed".to_string(), "1".to_string(), String::new()),
        ]
    );
    // the standing key rides ENTRY and RESOLVED only, and is the key the host served at the start
    assert_eq!(driven.scan_standing, ["", "1", "1", ""]);
    assert_eq!(driven.scan_log.len(), 4, "the roster events are not scan questions");
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);
    // a relink between walks: the old key is removed, a new one is issued and served, and the old one is never named again
    let (driven, row) = scan_row(", questions = 3", "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!((row["scan_after"].clone(), row["roster_after"].clone()), (Value::from(12), Value::from(6)));
    assert_eq!(
        (row["roster_insert"].clone(), row["roster_remove"].clone(), row["roster_reset"].clone()),
        (Value::from(3), Value::from(2), Value::from(1))
    );
    assert_eq!(
        driven
            .roster_log
            .iter()
            .map(|(_, result, name, identity)| (result.as_str(), name.as_str(), identity.as_str()))
            .collect::<Vec<_>>(),
        [
            ("cleared", "", ""),
            ("placed", "1", ""),
            ("removed", "", "1"),
            ("placed", "2", ""),
            ("removed", "", "2"),
            ("placed", "3", "")
        ]
    );
    assert_eq!(driven.scan_standing, ["", "1", "1", "", "", "2", "2", "", "", "3", "3", ""]);
    // a hosted mobile's insertion names its semantic identity in the same place; the real reply is still the key
    let (driven, row) = scan_row(", roster_mobile = true", "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!((row["request_faults"].clone(), row["reply_faults"].clone()), (Value::from(0), Value::from(0)));
    assert_eq!(
        driven.roster_log[1],
        ("0x40800000".to_string(), "placed".to_string(), "1".to_string(), "eech-world:1".to_string())
    );
    // the same walk under the paired controls: the events are never altered and a hit ends the scan with the standing key still joined
    for mode in ["scan_real", "scan_none"] {
        let (driven, row) = scan_row("", mode);
        assert!(driven.result.is_ok(), "{mode}: {:?}\n{}", driven.result, driven.rows);
        assert_eq!(
            (row["request_faults"].clone(), row["reply_faults"].clone()),
            (Value::from(0), Value::from(0)),
            "{mode}"
        );
        assert_eq!(row["roster_after"], 2, "{mode}");
    }
}

#[test]
fn only_roster_events_is_unreached_never_a_scan_and_a_roster_event_in_the_original_profile_is_not_the_original() {
    let driven = drive(&healthy(", only_roster_events = true"), &[]);
    let row = authority(&driven);
    assert_eq!((row["scan_after"].clone(), row["roster_after"].clone()), (Value::from(0), Value::from(2)));
    let error = error_of(driven);
    assert!(error.contains("UNREACHED") && error.contains("asked no entity_scan question"), "{error}");
    // a hosted-profile run is never credited for roster events alone, whatever the mode
    for mode in ["scan_real", "scan_none"] {
        let error = error_of(drive(&healthy(", only_roster_events = true"), &[("mode", mode)]));
        assert!(error.contains("UNREACHED"), "{mode}: {error}");
    }
    let error = error_of(drive(&healthy(", fixture_api = true, only_roster_events = true"), &[("profile", "original")]));
    assert!(error.contains("2 roster events") && error.contains("the module is not the original"), "{error}");
}

#[test]
fn each_single_fault_of_the_new_scan_and_roster_grammar_is_counted_at_its_question_and_is_never_a_pass() {
    // (the scenario, the prefix of the first fault, the fragment it names): a removal needs a second walk, because the first walk only boots and inserts
    let cases: [(&str, &str, &str); 18] = [
        (", bad_scan_request = 'no_key'", "scan request:", "names no roster key"),
        (", bad_scan_request = 'bad_key'", "scan request:", "is not a roster key"),
        (", bad_scan_request = 'begin_key'", "scan request:", "carries no standing key"),
        (", bad_scan_request = 'other_key'", "scan request:", "is not the entry the host last served"),
        (", questions = 2, bad_scan_request = 'ended_key'", "scan request:", "has ended"),
        (", bad_roster_request = 'phase7'", "scan request:", "is not 0 to 16"),
        (
            ", bad_roster_request = 'insert_stray_word'",
            "scan request:",
            "states only the position, the type and the side length",
        ),
        (", questions = 2, bad_roster_request = 'remove_no_key'", "scan request:", "a removal names"),
        (", questions = 2, bad_roster_request = 'remove_text'", "scan request:", "a removal names"),
        (", bad_roster_request = 'reset_identity'", "scan request:", "a boot carries no identity"),
        (", bad_roster_request = 'insert_long_identity'", "scan request:", "outside the roster's domain"),
        (", bad_roster_request = 'trailing_space'", "scan request:", "empty identity"),
        (", bad_real_roster = 'state'", "scan reply:", "placed"),
        (", bad_real_roster = 'no_key'", "scan reply:", "names no key"),
        (", bad_real_roster = 'bad_key'", "scan reply:", "is not a roster key"),
        (
            ", bad_real_roster = 'key_on_remove', questions = 2",
            "scan reply:",
            "a removal names is not a roster key",
        ),
        (", questions = 2, bad_real_roster = 'reused_key'", "scan reply:", "issued before"),
        (", questions = 2, bad_real_scan = 'ended_key_served'", "scan reply:", "has ended"),
    ];
    for (extra, prefix, fragment) in cases {
        let driven = drive(&healthy(extra), &[]);
        let row = authority(&driven);
        let faults = row["request_faults"].as_i64().unwrap() + row["reply_faults"].as_i64().unwrap();
        assert!(faults >= 1, "{extra}: {row}");
        let first = row["first_fault"].as_str().unwrap();
        assert!(first.starts_with(prefix) && first.contains(fragment), "{extra}: {first}");
        assert!(driven.result.is_err(), "{extra}: a malformed request or reply is never a pass");
    }
    // a valid state of another phase and a name with a state that serves none are the host's reply faults too
    for (extra, fragment) in [
        (", bad_real_scan = 'name_on_offer'", "a name with offer"),
        (", bad_real_scan = 'offer_for_resolved'", "is not what the kernel decides"),
    ] {
        let driven = drive(&healthy(extra), &[]);
        let row = authority(&driven);
        assert!(row["reply_faults"].as_i64().unwrap() >= 1, "{extra}: {row}");
        let first = row["first_fault"].as_str().unwrap();
        assert!(first.starts_with("scan reply:") && first.contains(fragment), "{extra}: {first}");
        assert!(driven.result.is_err(), "{extra}");
    }
}

/// The host keeps a city's record for the box it built and names it in the answer of the removal that leaves it: a roster key in a removal's answer is the contract, not a fault. The driver completes, counts no fault and
/// leaves every answer unaltered (the malformed name above is the single fault).
#[test]
fn a_removal_that_names_the_record_the_host_keeps_for_a_citys_box_is_no_fault() {
    let driven = drive(&healthy(", bad_real_roster = 'record_on_remove', questions = 2"), &[]);
    let row = authority(&driven);
    assert_eq!(row["request_faults"].as_i64().unwrap(), 0, "{row}");
    assert_eq!(row["reply_faults"].as_i64().unwrap(), 0, "{row}");
    assert!(driven.result.is_ok(), "{row}");
}

#[test]
fn the_paired_scan_control_reaches_the_candidate_dialogue_and_the_narrow_phase_with_the_configured_hit_and_the_cohort_physically_hits() {
    let natural = drive(&healthy(""), &[]);
    let (paired, row) = scan_row("", "scan_real");
    assert!(paired.result.is_ok(), "{:?}\n{}", paired.result, paired.rows);
    assert_eq!(row["completed"], true);
    // the enumeration is the host's real one; only the configured narrow-phase hit is held, and it is declared; a hit ends the walk with done_hit
    assert_eq!(
        (row["scan_altered"].clone(), row["scan_offer"].clone(), row["scan_done_hit"].clone()),
        (Value::from(0), Value::from(1), Value::from(1))
    );
    assert_eq!(
        (
            row["candidate_after"].clone(),
            row["entity_after"].clone(),
            row["answered_hit"].clone(),
            row["altered"].clone()
        ),
        (Value::from(1), Value::from(1), Value::from(1), Value::from(1))
    );
    assert_eq!(row["contact_any_hits_delta"], 1, "the actual caller used the configured hit");
    assert!(row["secondary_input"].as_str().unwrap().contains("configured_not_natural"), "{row}");
    assert!(row["cohort_signature"].as_str().unwrap().contains("target"), "the cohort physically hit");
    assert_ne!(
        row["cohort_signature"],
        authority(&natural)["cohort_signature"],
        "and that is not the natural healthy signature"
    );
    // a hit the actual caller did not use is not authority
    let error = error_of(drive(&healthy(", hit_ignored = true"), &[("mode", "scan_real")]));
    assert!(
        error.contains("scan_real: the configured hit never reached the weapon step's entity hit count"),
        "{error}"
    );
    // a real enumeration that never led to a narrow phase leaves the paired variant with nothing to prevent
    assert!(error_of(drive(&healthy(", scan_offers_nothing = true"), &[("mode", "scan_real")])).contains("UNREACHED"));
}

#[test]
fn the_empty_enumeration_variant_changes_the_cohort_by_preventing_every_candidate_question_narrow_phase_question_and_hit_with_the_same_configured_secondary_input(
) {
    let (paired, paired_row) = scan_row("", "scan_real");
    assert!(paired.result.is_ok());
    for extra in ["", ", questions = 3"] {
        let (variant, row) = scan_row(extra, "scan_none");
        assert!(variant.result.is_ok(), "{extra}: {:?}\n{}", variant.result, variant.rows);
        let asked = if extra.is_empty() { 1 } else { 3 };
        assert_eq!(
            (
                row["scan_after"].clone(),
                row["scan_altered"].clone(),
                row["scan_answered_done_none"].clone(),
                row["scan_phase_begin"].clone()
            ),
            (Value::from(asked), Value::from(asked), Value::from(asked), Value::from(asked)),
            "{extra}: every armed question was a start answered with an empty enumeration"
        );
        assert_eq!(
            (
                row["candidate_after"].clone(),
                row["entity_after"].clone(),
                row["altered"].clone(),
                row["contact_any_hits_delta"].clone()
            ),
            (Value::from(0), Value::from(0), Value::from(0), Value::from(0)),
            "{extra}"
        );
        assert_eq!(
            row["secondary_input"], paired_row["secondary_input"],
            "the same configured secondary input is declared in both paired modes"
        );
        assert_eq!(row["completed"], true);
        assert_ne!(row["cohort_signature"], paired_row["cohort_signature"], "{extra}");
        assert!(row["cohort_signature"].as_str().unwrap().contains("ground"), "{extra}");
        assert!(variant.delegation_broken.is_empty(), "{:?}", variant.delegation_broken);
    }
    // a campaign that goes on to offer an entry although the host ended the scan is a failure, not a pass
    let error = error_of(drive(&healthy(", ignore_none = true"), &[("mode", "scan_none")]));
    assert!(
        error.contains("still asked 1 entity_candidate, 1 entity_collision and 1 entity_pose questions after every enumeration was empty"),
        "{error}"
    );
    // no scan question at all is UNREACHED for the variant too
    let error = error_of(drive(&healthy(", no_entity_questions = true"), &[("mode", "scan_none")]));
    assert!(error.contains("UNREACHED") && error.contains("asked no entity_scan question"), "{error}");
}

#[test]
fn a_withheld_or_malformed_scan_answer_counts_only_at_the_anchored_engine_stop_with_its_exact_scan_marker_and_nothing_is_asked_afterwards() {
    for (mode, marker) in [
        ("withheld_scan", "entity_scan.callback"),
        ("bad_scan_state", "entity_scan.state"),
        ("bad_scan_cursor", "entity_scan.cursor"),
    ] {
        let driven = drive(&healthy(""), &[("mode", mode)]);
        let outcome = driven.result.clone();
        assert!(outcome.is_ok(), "{mode}: {outcome:?}\n{}", driven.rows);
        let row = authority(&driven);
        assert_eq!(row["completed"], false, "{mode}");
        assert_eq!(
            row["failure_marker"], marker,
            "{mode}: the scan's own marker, never the candidate's or the narrow phase's"
        );
        assert_eq!(
            (row["scan_altered"].clone(), row["candidate_altered"].clone(), row["altered"].clone()),
            (Value::from(1), Value::from(0), Value::from(0)),
            "{mode}"
        );
        assert_eq!(
            (row["scan_after"].clone(), row["scan_phase_begin"].clone()),
            (Value::from(1), Value::from(1)),
            "{mode}: the first armed question, the start of a scan, and nothing after the fatal"
        );
        assert_eq!(
            (row["candidate_after"].clone(), row["entity_after"].clone()),
            (Value::from(0), Value::from(0)),
            "{mode}: nothing downstream"
        );
        assert_eq!(row["case_weapons_pending"], -1, "{mode}: the cohort is not counted after the stop");
        assert_eq!(row["contact_any_tests_delta"], -1, "{mode}: no counter is read from a stopped engine");
        assert_eq!(row["contact_any_hits_delta"], -1, "{mode}: unknown stays unknown");
    }
    // the state is illegal only at the start of a scan, and the cursor is altered only from a real first: both are the eligible question, not a callback setup
    let row = authority(&drive(&healthy(""), &[("mode", "bad_scan_cursor")]));
    assert_eq!(row["scan_first"], 1, "the real first sector was the eligible answer");
    // the old markers are not credited to a scan mode, and a witness failure cannot pose as the host withholding an answer
    for old in ["entity_candidate.callback", "entity_collision.callback"] {
        let error = error_of(drive(
            &healthy(&format!(", engine_stop = {{ after = 1, marker = '{old}' }}")),
            &[("mode", "withheld_scan")],
        ));
        assert!(
            error.contains(&format!("stopped at [model-provider:{old}], not at [model-provider:entity_scan.callback]")),
            "{old}: {error}"
        );
    }
    for scenario in [
        "engine_stop = { after = 1, marker = 'entity_scan.callback', text_only = true }",
        "engine_stop = { after = 1, marker = 'x', wrapper = 'eech: EECH fatal error: host model: [model-provider:entity_scan.callback] x' }",
    ] {
        let error = error_of(drive(&healthy(&format!(", {scenario}")), &[("mode", "withheld_scan")]));
        assert!(error.contains("an unrecognised failure, not the engine's known fatal"), "{scenario}: {error}");
    }
    // a stop at the scan's own marker that the provider did not cause (nothing altered) is not credited, and a flight that never asks the host completes without the stop: a failure
    let error = error_of(drive(
        &healthy(", engine_stop = { after = 1, marker = 'entity_scan.callback' }"),
        &[("mode", "withheld_scan")],
    ));
    assert!(error.contains("without the provider having altered"), "{error}");
    let error = error_of(drive(&healthy(", no_entity_questions = true"), &[("mode", "withheld_scan")]));
    assert!(error.contains("the engine did not stop at entity_scan.callback"), "{error}");
    // a malformed REAL reply is never altered into a pass: the witness counts it and the mode does not stop, which is a failure and not a refusal
    let driven = drive(&healthy(", bad_real_scan = 'name'"), &[("mode", "withheld_scan")]);
    let row = authority(&driven);
    assert!(row["reply_faults"].as_i64().unwrap() >= 1, "{row}");
    assert_eq!((row["scan_altered"].clone(), row["failure_marker"].clone()), (Value::from(0), Value::from("")));
    assert!(error_of(driven).contains("the engine did not stop at entity_scan.callback"));
}

#[test]
fn the_original_profile_has_the_api_and_the_real_caller_lookups_but_no_scan_candidate_or_narrow_phase_question_and_a_missing_api_is_unreached() {
    let original = [("profile", "original")];
    let driven = drive(&healthy(", fixture_api = true, original_tests = true"), &original);
    let row = authority(&driven);
    assert_eq!(
        (
            row["scan_after"].clone(),
            row["candidate_after"].clone(),
            row["entity_after"].clone(),
            row["fixture_api"].clone()
        ),
        (Value::from(0), Value::from(0), Value::from(0), Value::from(true))
    );
    let error = error_of(driven);
    assert!(
        error.contains("CAPABILITY RED") && error.contains("no entity_scan question") && error.contains("fixture API present"),
        "{error}"
    );
    // a module that asked the host a scan question is not the original
    let error = error_of(drive(&healthy(", fixture_api = true"), &original));
    assert!(error.contains("entity_pose questions: the module is not the original"), "{error}");
    // a missing fixture API and a caller that never ran are UNREACHED, never a red
    assert!(error_of(drive(&healthy(", original_tests = true"), &original)).contains("collision-fixture API"));
    assert!(error_of(drive(&healthy(", fixture_api = true, no_entity_tests = true"), &original)).contains("contact_any_tests did not rise"));
    // the scan modes are hosted-profile modes only
    for mode in [
        "scan_real",
        "scan_none",
        "withheld_scan",
        "bad_scan_state",
        "bad_scan_cursor",
        "withdrawn_backing",
    ] {
        assert!(
            error_of(drive("", &[("profile", "original"), ("mode", mode)])).contains("only mode=control"),
            "{mode}"
        );
    }
}

#[test]
fn a_standing_key_that_fails_the_request_join_is_a_request_fault_and_is_never_altered_or_credited_as_a_scan_refusal() {
    // positive first: the valid withheld_scan is still recognised at its first armed scan question, which is the start of a scan and carries no standing key
    let driven = drive(&healthy(""), &[("mode", "withheld_scan")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = authority(&driven);
    assert_eq!(
        (row["failure_marker"].clone(), row["scan_altered"].clone(), row["request_faults"].clone()),
        (Value::from("entity_scan.callback"), Value::from(1), Value::from(0))
    );
    // the first armed question the wrapper sees is ENTRY, standing at a well-formed roster key that is not the entry the host last served, or one that has ended: a request fault at that question, so no
    // alteration, no engine stop and no refusal credit
    for (walk, fragment) in [("last_served", "which is not the entry the host last served"), ("ended", "which has ended")] {
        let driven = drive(&healthy(&format!(", scan_from_entry = '{walk}'")), &[("mode", "withheld_scan")]);
        let row = authority(&driven);
        assert_eq!(
            (row["scan_altered"].clone(), row["failure_marker"].clone()),
            (Value::from(0), Value::from("")),
            "{walk}: the malformed question was altered"
        );
        assert!(row["request_faults"].as_i64().unwrap() >= 1, "{walk}: {row}");
        let first = row["first_fault"].as_str().unwrap();
        assert!(first.starts_with("scan request:") && first.contains(fragment), "{walk}: {first}");
        let error = error_of(driven);
        assert!(
            error.contains("the engine did not stop at entity_scan.callback"),
            "{walk}: a malformed input is never a credited refusal: {error}"
        );
    }
}

#[test]
fn the_real_provider_serves_the_scan_and_withdrawing_the_served_backing_by_its_real_remove_makes_the_next_entry_refused_by_name() {
    // the healthy control first: the REAL provider (its own sector roster) answers every scan and roster question, nothing is altered, and the whole chain is reached
    let (driven, row) = scan_row(", real_scan = true", "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(
        (row["request_faults"].clone(), row["reply_faults"].clone(), row["roster_withdrawn"].clone()),
        (Value::from(0), Value::from(0), Value::from(0))
    );
    assert_eq!(
        (row["scan_after"].clone(), row["roster_after"].clone(), row["scan_altered"].clone()),
        (Value::from(4), Value::from(2), Value::from(0))
    );
    assert_eq!(
        driven
            .roster_log
            .iter()
            .map(|(_, result, name, _)| (result.as_str(), name.as_str()))
            .collect::<Vec<_>>(),
        [("cleared", ""), ("placed", "1")]
    );
    assert_eq!(driven.scan_standing, ["", "1", "1", ""]);
    assert_eq!(
        driven.scan_log.iter().map(|(_, result)| result.as_str()).collect::<Vec<_>>(),
        ["first", "offer", "next", "first"]
    );

    // the mode: at the first real FIRST that names an entry the host calls the SAME real provider's REMOVE on that key, returns the FIRST reply unchanged, and the next real ENTRY is refused by name
    let driven = drive(&healthy(", real_scan = true"), &[("mode", "withdrawn_backing")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = authority(&driven);
    assert_eq!(row["completed"], false);
    assert_eq!(row["failure_marker"], "entity_scan.refused");
    assert_eq!(
        (
            row["roster_withdrawn"].clone(),
            row["roster_withdrawn_next_phase"].clone(),
            row["roster_withdrawn_cause"].clone()
        ),
        (Value::from(1), Value::from("phase_entry"), Value::from("roster_key_ended")),
        "{row}"
    );
    // nothing else was altered: no scan, candidate or narrow-phase answer, and the campaign's own roster events are only its boot and its insertion
    assert_eq!(
        (
            row["scan_altered"].clone(),
            row["candidate_altered"].clone(),
            row["altered"].clone(),
            row["request_faults"].clone()
        ),
        (Value::from(0), Value::from(0), Value::from(0), Value::from(0))
    );
    assert_eq!(
        (row["roster_after"].clone(), row["candidate_after"].clone(), row["entity_after"].clone()),
        (Value::from(2), Value::from(0), Value::from(0))
    );
    // the only reply fault is the host's own named refusal of the ended entry, never another
    assert_eq!(row["reply_faults"], 1);
    assert_eq!(row["first_fault"], "scan reply: result refused");
    // the real provider acknowledged the REAL remove of the served key once, in the order: boot, insertion, removal; the first reply was handed on untouched
    assert_eq!(
        driven
            .roster_log
            .iter()
            .map(|(phase, result, name, identity)| (phase.as_str(), result.as_str(), name.as_str(), identity.as_str()))
            .collect::<Vec<_>>(),
        [
            ("0x40c00000", "cleared", "", ""),
            ("0x40800000", "placed", "1", ""),
            ("0x40a00000", "removed", "", "1")
        ]
    );
    assert_eq!(
        driven.scan_log.iter().map(|(_, result)| result.as_str()).collect::<Vec<_>>(),
        ["first", "refused"]
    );
    assert_eq!(driven.scan_standing, ["", "1"], "the continuation stood at the key the first reply served");
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);

    // the mode is credited only by the reached withdrawal and the named refusal: a provider that does not refuse the ended entry (the scripted one), or an engine stop elsewhere, is never a pass
    let error = error_of(drive(&healthy(""), &[("mode", "withdrawn_backing")]));
    assert!(error.contains("completed: the engine did not stop at entity_scan.refused"), "{error}");
    let error = error_of(drive(
        &healthy(", real_scan = true, engine_stop = { after = 1, marker = 'entity_candidate.callback' }"),
        &[("mode", "withdrawn_backing")],
    ));
    assert!(error.contains("not at [model-provider:entity_scan.refused]"), "{error}");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the CITY events (phases 7 to 12): a city's buildings are the host's list. The real provider serves them; the driver counts them apart from the scan's and the roster's events, joins their keys, and the
// `withdrawn_child` mode shows the traversal is served from that list (the campaign's own list at the same place cannot be what answers)

/// a city candidate: the real provider answers every scan, roster and city question; the city's sector placement lists three buildings; the candidate's `buildings` state walks them
const CITY: &str = ", real_scan = true, fixed_candidate = true, city = 3, candidate_sequence = { 'need_fixed', 'buildings', 'test' }";

#[test]
fn a_city_traversal_served_by_the_real_provider_is_a_healthy_control_with_the_city_events_counted_apart() {
    let (driven, row) = scan_row(CITY, "scan_real");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(
        (row["request_faults"].clone(), row["reply_faults"].clone()),
        (Value::from(0), Value::from(0)),
        "{row}"
    );
    // the city's three buildings are listed (head first: the newest is served first), and the traversal asks for the first and then after each; nothing else changed
    for (field, expected) in [
        ("city_after", 7),
        ("city_child_insert", 3),
        ("city_child_first", 1),
        ("city_child_next", 3),
        ("city_next_joined", 3),
        ("city_placed", 3),
        ("city_next", 4),
        ("city_served", 3),
        ("city_withdrawn", 0),
        ("roster_after", 2),
        // the configured narrow-phase hit of the paired control ends the scan: the start, the served entry and the hit
        ("scan_after", 3),
        ("scan_altered", 0),
    ] {
        assert_eq!(row[field], expected, "{field}: {row}");
    }
    let log: Vec<(&str, &str, &str, &str)> = driven
        .city_log
        .iter()
        .map(|(phase, result, name, identity)| (phase.as_str(), result.as_str(), name.as_str(), identity.as_str()))
        .collect();
    assert_eq!(
        log,
        [
            ("0x40e00000", "placed", "2", "1"),
            ("0x40e00000", "placed", "3", "1"),
            ("0x40e00000", "placed", "4", "1"),
            ("0x41100000", "next", "4", "1"),
            ("0x41200000", "next", "3", "1 4"),
            ("0x41200000", "next", "2", "1 3"),
            ("0x41200000", "next", "", "1 2"),
        ]
    );
    assert_eq!(driven.scan_log.len(), 3, "the city events are not scan questions");
    assert_eq!(driven.roster_log.len(), 2, "and not roster events");
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);
    // a building that is also placed in its sector is declared the same entity and named by an object identity: the real provider acknowledges both
    let (driven, row) = scan_row(&format!("{CITY}, city_link = true"), "scan_real");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(
        (
            row["request_faults"].clone(),
            row["reply_faults"].clone(),
            row["city_link"].clone(),
            row["city_name"].clone()
        ),
        (Value::from(0), Value::from(0), Value::from(1), Value::from(1))
    );
    let phases: Vec<&str> = driven.city_log.iter().map(|(phase, ..)| phase.as_str()).collect();
    assert_eq!(&phases[3..5], ["0x41300000", "0x41400000"]);
    assert_eq!(driven.city_log[3].3, "5 2", "the sector placement and the first building listed");
    assert_eq!(driven.city_log[4].3, "5 eech-world:object:7");
    // two walks: the buildings leave their city before the city's own placement is relinked, and every key of the first walk ends
    let (driven, row) = scan_row(&format!("{CITY}, questions = 2"), "scan_real");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(
        (
            row["request_faults"].clone(),
            row["reply_faults"].clone(),
            row["city_child_remove"].clone(),
            row["city_child_insert"].clone()
        ),
        (Value::from(0), Value::from(0), Value::from(3), Value::from(6))
    );
}

#[test]
fn withdrawing_the_served_building_by_the_hosts_real_remove_makes_the_next_building_question_refused_by_name() {
    let driven = drive(&healthy(CITY), &[("mode", "withdrawn_child")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = authority(&driven);
    assert_eq!(row["completed"], false);
    assert_eq!(row["failure_marker"], "entity_scan.refused");
    assert_eq!(
        (
            row["city_withdrawn"].clone(),
            row["city_withdrawn_next"].clone(),
            row["city_withdrawn_cause"].clone()
        ),
        (Value::from(1), Value::from("child_next"), Value::from("city_child_ended")),
        "{row}"
    );
    // nothing was altered, the campaign's request was joined by the driver (it does not know the host removed the building), and the only fault is the host's own named refusal
    assert_eq!(
        (
            row["scan_altered"].clone(),
            row["candidate_altered"].clone(),
            row["altered"].clone(),
            row["request_faults"].clone(),
            row["reply_faults"].clone()
        ),
        (Value::from(0), Value::from(0), Value::from(0), Value::from(0), Value::from(1))
    );
    assert_eq!(row["first_fault"], "scan reply: result refused");
    // the campaign asked the first building and then the next one; the removal (the host's own, uncounted among the campaign's events) was acknowledged between them by the same real provider
    assert_eq!(
        (
            row["city_after"].clone(),
            row["city_child_first"].clone(),
            row["city_child_next"].clone(),
            row["city_served"].clone()
        ),
        (Value::from(5), Value::from(1), Value::from(1), Value::from(1))
    );
    let log: Vec<(&str, &str, &str, &str)> = driven
        .city_log
        .iter()
        .map(|(phase, result, name, identity)| (phase.as_str(), result.as_str(), name.as_str(), identity.as_str()))
        .collect();
    assert_eq!(
        &log[3..],
        [
            ("0x41100000", "next", "4", "1"),
            ("0x41000000", "removed", "", "1 4"),
            ("0x41200000", "refused", "", "1 4"),
        ]
    );
    assert_eq!(row["candidate_after"], 2, "need_fixed and buildings: nothing after the stop");
    assert_eq!(row["entity_after"], 0, "no narrow phase");
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);
    // a case with no city traversal, or one that never asks for a building after a served one, has no continuation to withdraw before: a named finite prerequisite, never a pass; and neither is an engine stop elsewhere
    let error = error_of(drive(&healthy(", real_scan = true"), &[("mode", "withdrawn_child")]));
    assert!(error.contains("UNREACHED") && error.contains("holds no such traversal"), "{error}");
    let error = error_of(drive(
        &healthy(&format!("{CITY}, engine_stop = {{ after = 1, marker = 'entity_candidate.callback' }}")),
        &[("mode", "withdrawn_child")],
    ));
    assert!(error.contains("UNREACHED"), "{error}");
    // the Original profile asks the host no city question
    let error = error_of(drive(&healthy(", fixture_api = true"), &[("profile", "original"), ("mode", "withdrawn_child")]));
    assert!(error.contains("only mode=control"), "{error}");
}

#[test]
fn a_traversal_that_never_asks_for_the_next_building_is_unreached_and_a_question_that_does_not_join_is_never_withdrawn_before() {
    // the real caller asks for the first building before the city's bounding box decision and for the next one only after a building's own collision attempt, which can end the traversal: a healthy control
    // with FIRST only is valid, and shows no actual next-building question
    let (driven, row) = scan_row(&format!("{CITY}, city_first_only = true"), "scan_real");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    for (field, expected) in [
        ("city_child_first", 1),
        ("city_child_next", 0),
        ("city_next_joined", 0),
        ("city_served", 1),
        ("request_faults", 0),
        ("reply_faults", 0),
    ] {
        assert_eq!(row[field], expected, "{field}: {row}");
    }
    // the withdrawal needs an actual next-building question, so with none it is UNREACHED, never a pass, and the host's list is not touched: no removal was ever sent
    let driven = drive(&healthy(&format!("{CITY}, city_first_only = true")), &[("mode", "withdrawn_child")]);
    let error = error_of(driven);
    assert!(
        error.contains("UNREACHED") && error.contains("asked for no building after a served one"),
        "{error}"
    );
    let driven = drive(&healthy(&format!("{CITY}, city_first_only = true")), &[("mode", "withdrawn_child")]);
    let row = authority(&driven);
    assert_eq!(
        (row["city_withdrawn"].clone(), row["city_removed"].clone()),
        (Value::from(0), Value::from(0)),
        "{row}"
    );
    assert!(driven.city_log.iter().all(|(phase, ..)| phase != "0x41000000"), "{:?}", driven.city_log);
    // a next-building question that does not stand at the building the host last served does not join the keys seen: it is a request fault, nothing is withdrawn before it, and it is not the continuation
    let driven = drive(
        &healthy(&format!("{CITY}, bad_city_request = 'next_not_last_served'")),
        &[("mode", "withdrawn_child")],
    );
    let row = authority(&driven);
    assert_eq!(
        (row["city_withdrawn"].clone(), row["city_next_joined"].clone()),
        (Value::from(0), Value::from(0)),
        "{row}"
    );
    assert!(row["request_faults"].as_i64().unwrap() >= 1, "{row}");
    let error = error_of(driven);
    assert!(
        !error.contains("UNREACHED") && error.contains("not the building the host last served"),
        "{error}"
    );
    assert!(
        drive(
            &healthy(&format!("{CITY}, bad_city_request = 'next_not_last_served'")),
            &[("mode", "withdrawn_child")]
        )
        .city_log
        .iter()
        .all(|(phase, ..)| phase != "0x41000000"),
        "no withdrawal was sent before a question that does not join"
    );
}

#[test]
fn a_citys_sector_relink_with_its_buildings_listed_keeps_the_list_name_and_the_traversal_and_the_withdrawal_still_work() {
    // the city's placement is removed and inserted again while its three buildings stay listed, and declared to continue the list: the real provider acknowledges it, serves the same buildings in the same
    // order under the name they were listed by (an ended placement's key, which the driver must not call an ended city), and nothing is refused
    let (driven, row) = scan_row(&format!("{CITY}, city_relink = true"), "scan_real");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    for (field, expected) in [
        ("request_faults", 0),
        ("reply_faults", 0),
        ("city_link", 1),
        ("city_child_first", 1),
        ("city_child_next", 3),
        ("city_next_joined", 3),
        ("city_served", 3),
    ] {
        assert_eq!(row[field], expected, "{field}: {row}");
    }
    let log: Vec<(&str, &str, &str, &str)> = driven
        .city_log
        .iter()
        .map(|(phase, result, name, identity)| (phase.as_str(), result.as_str(), name.as_str(), identity.as_str()))
        .collect();
    assert_eq!(
        &log[3..],
        [
            ("0x41300000", "placed", "1", "5 1"),
            ("0x41100000", "next", "4", "1"),
            ("0x41200000", "next", "3", "1 4"),
            ("0x41200000", "next", "2", "1 3"),
            ("0x41200000", "next", "", "1 2"),
        ],
        "the list keeps the name 1 across the relink (the city's new placement is 5)"
    );
    // two walks: the buildings are unlisted under the same name, and the city's relinked placement then leaves
    let (driven, row) = scan_row(&format!("{CITY}, city_relink = true, questions = 2"), "scan_real");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(
        (row["request_faults"].clone(), row["reply_faults"].clone(), row["city_child_remove"].clone()),
        (Value::from(0), Value::from(0), Value::from(3)),
        "{row}"
    );
    // the withdrawal at the next-building question of the same relinked city
    let driven = drive(&healthy(&format!("{CITY}, city_relink = true")), &[("mode", "withdrawn_child")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = authority(&driven);
    assert_eq!(
        (
            row["city_withdrawn"].clone(),
            row["city_withdrawn_cause"].clone(),
            row["request_faults"].clone(),
            row["reply_faults"].clone()
        ),
        (Value::from(1), Value::from("city_child_ended"), Value::from(0), Value::from(1)),
        "{row}"
    );
}

#[test]
fn a_citys_list_name_survives_the_last_building_leaving_a_building_listed_again_and_a_second_relink() {
    // relink, every building leaves while the city stays placed, one is listed again under the list's name, relink again, then the traversal and (two walks) the unlisting: all by the record's name 1, with the
    // real provider, and nothing refused
    let (driven, row) = scan_row(&format!("{CITY}, city_relink = true, city_cycle = true"), "scan_real");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    for (field, expected) in [
        ("request_faults", 0),
        ("reply_faults", 0),
        ("city_child_insert", 4),
        ("city_child_remove", 3),
        ("city_link", 2),
        ("city_child_first", 1),
        ("city_child_next", 1),
        ("city_served", 1),
    ] {
        assert_eq!(row[field], expected, "{field}: {row}");
    }
    let log: Vec<(&str, &str, &str, &str)> = driven
        .city_log
        .iter()
        .map(|(phase, result, name, identity)| (phase.as_str(), result.as_str(), name.as_str(), identity.as_str()))
        .collect();
    let links: Vec<&str> = log.iter().filter(|entry| entry.0 == "0x41300000").map(|entry| entry.3).collect();
    assert_eq!(links, ["5 1", "7 1"], "both relinks continue the record by its first name: {log:?}");
    assert_eq!(&log[log.len() - 2..], [("0x41100000", "next", "6", "1"), ("0x41200000", "next", "", "1 6")]);
    let (driven, row) = scan_row(&format!("{CITY}, city_relink = true, city_cycle = true, questions = 2"), "scan_real");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(
        (row["request_faults"].clone(), row["reply_faults"].clone()),
        (Value::from(0), Value::from(0)),
        "{row}"
    );
}

#[test]
fn a_city_name_carries_an_object_identity_of_its_whole_domain_beside_its_key_and_nothing_beyond_it() {
    // the object identity has its own domain of 1 to 128 bytes beside the placement key: a 128 byte identity and the key that names its placement cross the driver's grammar and the real provider whole (the
    // place is longer than the 128 bytes a host identity may be), and one more byte is the driver's named request fault, never a pass
    let (driven, row) = scan_row(&format!("{CITY}, city_link = true, city_object_length = 128"), "scan_real");
    assert!(
        driven.result.is_ok(),
        "{:?}
{}",
        driven.result,
        driven.rows
    );
    assert_eq!(
        (row["request_faults"].clone(), row["reply_faults"].clone(), row["city_name"].clone()),
        (Value::from(0), Value::from(0), Value::from(1)),
        "{row}"
    );
    let named = driven
        .city_log
        .iter()
        .find(|(phase, ..)| phase == "0x41400000")
        .expect("the NAME reached the real provider");
    assert_eq!((named.1.as_str(), named.3.len()), ("placed", "5 ".len() + 128), "{named:?}");
    let driven = drive(
        &healthy(&format!("{CITY}, city_link = true, city_object_length = 129")),
        &[("mode", "scan_real")],
    );
    let row = authority(&driven);
    assert!(row["request_faults"].as_i64().unwrap() >= 1, "{row}");
    let first = row["first_fault"].as_str().unwrap();
    assert!(
        first.starts_with("scan request:") && first.contains("129 bytes") && first.contains("1 to 128"),
        "{first}"
    );
    assert!(driven.result.is_err(), "an identity beyond its domain is never a pass");
}

#[test]
fn each_single_fault_of_the_city_grammar_is_counted_at_its_question_and_is_never_a_pass() {
    // (the scenario, the prefix of the first fault, the fragment it names)
    let cases: [(&str, &str, &str); 14] = [
        (", bad_city_request = 'stray_fact'", "scan request:", "states nothing but its phase"),
        (", bad_city_request = 'no_city'", "scan request:", "which is not a roster key"),
        (", bad_city_request = 'two_keys_first'", "scan request:", "names the city only"),
        (", bad_city_request = 'one_key_next'", "scan request:", "names one key where it needs two"),
        (", bad_city_request = 'bad_key'", "scan request:", "which is not a roster key"),
        (
            ", bad_city_request = 'next_not_last_served'",
            "scan request:",
            "not the building the host last served",
        ),
        (", city_link = true, bad_city_request = 'name_empty'", "scan request:", "empty object identity"),
        (", city_link = true, bad_city_request = 'name_control'", "scan request:", "control character"),
        (", bad_real_city = 'state'", "scan reply:", "answered placed, not next"),
        (", bad_real_city = 'insert_no_key'", "scan reply:", "names no key"),
        (", bad_real_city = 'served_not_key'", "scan reply:", "is not a roster key"),
        (", bad_real_city = 'reused_key'", "scan reply:", "was issued before"),
        (", city_link = true, bad_real_city = 'link_echo'", "scan reply:", "acknowledged"),
        (", city_link = true, bad_real_city = 'name_echo'", "scan reply:", "acknowledged"),
    ];
    for (extra, prefix, fragment) in cases {
        let driven = drive(&healthy(&format!("{CITY}{extra}")), &[("mode", "scan_real")]);
        let row = authority(&driven);
        let faults = row["request_faults"].as_i64().unwrap() + row["reply_faults"].as_i64().unwrap();
        assert!(faults >= 1, "{extra}: {row}");
        let first = row["first_fault"].as_str().unwrap();
        assert!(first.starts_with(prefix) && first.contains(fragment), "{extra}: {first}");
        assert!(driven.result.is_err(), "{extra}: a malformed request or reply is never a pass");
    }
}

#[test]
fn the_scan_modes_close_the_evidence_with_a_checked_close_and_a_failing_close_is_never_hidden() {
    let driven = drive(&healthy(", io_fail = { op = 'close' }"), &[("mode", "scan_none")]);
    assert_eq!(driven.closed, 1);
    assert!(error_of(driven).contains("OUTPUT FAILURE"), "the close failure is raised");
}

// ---- ENTITY_POSE through the witness (EEDCS-49, D3, fifth slice): the observation of a mobile candidate is counted by stage and class, never altered outside its refusal modes, and never asserted ----

fn pose_row(extra: &str, mode: &str) -> (Driven, Value) {
    let driven = drive(&healthy(extra), &[("mode", mode)]);
    let row = authority(&driven);
    (driven, row)
}

#[test]
fn the_natural_control_counts_the_pose_observations_by_stage_and_class_and_keeps_the_histogram_with_nothing_altered() {
    let (driven, row) = pose_row("", "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    // one walk: the mobile candidate is observed once, at the offer, with a named identity, and the real answer is a recorded routed vehicle with no held motion vector (difference 8)
    assert_eq!((row["pose_before"].clone(), row["pose_after"].clone()), (Value::from(0), Value::from(1)));
    for (field, expected) in [
        ("pose_stage_offer", 1),
        ("pose_stage_placement", 0),
        ("pose_named", 1),
        ("pose_recorded", 1),
        ("pose_unrecorded", 0),
        ("pose_retired", 0),
        ("pose_unknown", 0),
        ("pose_no_id", 0),
        ("pose_altered", 0),
    ] {
        assert_eq!(row[field], expected, "{field}");
    }
    assert_eq!(
        row["pose_histogram"], "0:4:1:recorded:1:8=1",
        "stage, the entity's type, alive, class, record kind and difference"
    );
    assert_eq!(driven.pose_log.len(), 1);
    let (stage, entity_type, alive, result, key) = &driven.pose_log[0];
    assert_eq!((*stage, *entity_type, *alive, result.as_str()), (0, 4, 1, "recorded"));
    assert!(
        key.ends_with(" eech-world:1") && key.len() > 164,
        "the 15 patterns and the identity after one space: {key}"
    );
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);
    // the placement stage is a second observation, with the attitude
    let (driven, row) = pose_row(", candidate_sequence = { 'need_placement', 'test' }", "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(
        (row["pose_after"].clone(), row["pose_stage_offer"].clone(), row["pose_stage_placement"].clone()),
        (Value::from(2), Value::from(1), Value::from(1))
    );
    assert_eq!(row["pose_histogram"], "0:4:1:recorded:1:8=1;1:4:1:recorded:1:8=1");
    // three walks, three observations
    let (driven, row) = pose_row(", questions = 3", "control");
    assert!(driven.result.is_ok(), "{:?}", driven.result);
    assert_eq!(row["pose_after"], 3);
    assert_eq!(row["pose_histogram"], "0:4:1:recorded:1:8=3");
    // a class that is not a record, a record of another kind and another difference are OBSERVED and carried into the evidence, never asserted: the control still completes
    for (extra, class, histogram) in [
        (", pose_class = 'unknown'", "pose_unknown", "0:4:1:unknown:0:0=1"),
        (", pose_class = 'unrecorded'", "pose_unrecorded", "0:4:1:unrecorded:0:0=1"),
        (", pose_class = 'retired'", "pose_retired", "0:4:1:retired:0:0=1"),
        (", pose_kind = 4, pose_difference = 16", "pose_recorded", "0:4:1:recorded:4:16=1"),
        (", unnamed_pose = true", "pose_no_id", "0:4:1:no_id:0:0=1"),
    ] {
        let (driven, row) = pose_row(extra, "control");
        assert!(driven.result.is_ok(), "{extra}: {:?}\n{}", driven.result, driven.rows);
        assert_eq!(row[class], 1, "{extra}");
        assert_eq!(row["pose_histogram"], histogram, "{extra}");
        assert_eq!(row["pose_named"], if extra.contains("unnamed") { 0 } else { 1 }, "{extra}");
    }
}

#[test]
fn zero_pose_questions_is_unreached_and_a_malformed_pose_request_or_real_reply_is_counted_and_never_credited() {
    // the enumeration, the candidate dialogue and the narrow phase all happened but no observation reached the host: the mobile collaborator contract was not observed
    let driven = drive(&healthy(", no_pose_questions = true"), &[]);
    let row = authority(&driven);
    assert_eq!(
        (row["pose_after"].clone(), row["candidate_after"].clone(), row["entity_after"].clone()),
        (Value::from(0), Value::from(1), Value::from(1))
    );
    let error = error_of(driven);
    assert!(error.contains("UNREACHED") && error.contains("asked no entity_pose question"), "{error}");
    // a request that is not 15 patterns is counted
    let row = authority(&drive(&healthy(", bad_pose_request = 'count', unnamed_pose = true"), &[]));
    assert!(row["request_faults"].as_i64().unwrap() >= 1, "{row}");
    assert!(row["first_fault"].as_str().unwrap().starts_with("pose request:"), "{row}");
    // real replies that break the grammar are counted at the question
    for bad in ["state", "kind", "no_id_for_named", "kind_for_unrecorded"] {
        let driven = drive(&healthy(&format!(", bad_real_pose = '{bad}'")), &[]);
        let row = authority(&driven);
        assert!(row["reply_faults"].as_i64().unwrap() >= 1, "{bad}: {row}");
        assert!(row["first_fault"].as_str().unwrap().starts_with("pose reply:"), "{bad}: {row}");
        assert!(driven.result.is_err(), "{bad}: a malformed real reply is never a pass");
    }
    // the host itself refusing is the callback marker of the engine's stop, never a pass
    let driven = drive(&healthy(", real_pose_error = true"), &[]);
    assert!(driven.result.is_err(), "a provider that raises is never a pass");
}

#[test]
fn a_withheld_or_malformed_pose_answer_counts_only_at_the_anchored_engine_stop_with_its_exact_pose_marker_and_nothing_is_asked_afterwards() {
    for (mode, marker) in [
        ("withheld_pose", "entity_pose.callback"),
        ("bad_pose_answer", "entity_pose.answer"),
        ("bad_pose_state", "entity_pose.state"),
    ] {
        let driven = drive(&healthy(""), &[("mode", mode)]);
        let outcome = driven.result.clone();
        assert!(outcome.is_ok(), "{mode}: {outcome:?}\n{}", driven.rows);
        let row = authority(&driven);
        assert_eq!(row["completed"], false, "{mode}");
        assert_eq!(
            row["failure_marker"], marker,
            "{mode}: the pose's own marker, never the candidate's, the scan's or the narrow phase's"
        );
        assert_eq!(
            (
                row["pose_altered"].clone(),
                row["scan_altered"].clone(),
                row["candidate_altered"].clone(),
                row["altered"].clone()
            ),
            (Value::from(1), Value::from(0), Value::from(0), Value::from(0)),
            "{mode}"
        );
        assert_eq!(
            (row["pose_after"].clone(), row["pose_stage_offer"].clone()),
            (Value::from(1), Value::from(1)),
            "{mode}: the first armed observation, and nothing after the fatal"
        );
        assert_eq!(
            (row["candidate_after"].clone(), row["entity_after"].clone()),
            (Value::from(0), Value::from(0)),
            "{mode}: the observation is asked before the candidate's first question, so nothing downstream"
        );
        assert_eq!(row["case_weapons_pending"], -1, "{mode}: the cohort is not counted after the stop");
        assert_eq!(row["contact_any_tests_delta"], -1, "{mode}: no counter is read from a stopped engine");
        assert_eq!(row["contact_any_hits_delta"], -1, "{mode}: unknown stays unknown");
    }
    // the old markers are not credited to a pose mode, and a witness failure cannot pose as the host withholding an answer
    for old in ["entity_candidate.callback", "entity_scan.callback", "entity_collision.callback"] {
        let error = error_of(drive(
            &healthy(&format!(", engine_stop = {{ after = 1, marker = '{old}' }}")),
            &[("mode", "withheld_pose")],
        ));
        assert!(
            error.contains(&format!("stopped at [model-provider:{old}], not at [model-provider:entity_pose.callback]")),
            "{old}: {error}"
        );
    }
    for scenario in [
        "engine_stop = { after = 1, marker = 'entity_pose.callback', text_only = true }",
        "engine_stop = { after = 1, marker = 'x', wrapper = 'eech: EECH fatal error: host model: [model-provider:entity_pose.callback] x' }",
    ] {
        let error = error_of(drive(&healthy(&format!(", {scenario}")), &[("mode", "withheld_pose")]));
        assert!(error.contains("an unrecognised failure, not the engine's known fatal"), "{scenario}: {error}");
    }
    // a stop at the pose's own marker that the provider did not cause is not credited, and a flight that never asks the host completes without the stop: a failure
    let error = error_of(drive(
        &healthy(", engine_stop = { after = 1, marker = 'entity_pose.callback' }"),
        &[("mode", "withheld_pose")],
    ));
    assert!(error.contains("without the provider having altered"), "{error}");
    let error = error_of(drive(&healthy(", no_pose_questions = true"), &[("mode", "withheld_pose")]));
    assert!(error.contains("the engine did not stop at entity_pose.callback"), "{error}");
    // "no identity" is altered only for a question that NAMED one: an unnamed observation is not the eligible question, so the mode completes without its stop
    let error = error_of(drive(&healthy(", unnamed_pose = true"), &[("mode", "bad_pose_state")]));
    assert!(error.contains("the engine did not stop at entity_pose.state"), "{error}");
}

#[test]
fn the_pose_observation_follows_the_scan_the_empty_enumeration_prevents_it_and_the_original_profile_asks_none() {
    // the paired control reaches the candidate dialogue and therefore the observation; the empty enumeration prevents every downstream question, the observation included
    let (driven, row) = scan_row("", "scan_real");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(row["pose_after"], 1, "the paired control observes the mobile candidate");
    let (driven, row) = scan_row("", "scan_none");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(row["pose_after"], 0, "no candidate was offered, so nothing was observed");
    // a campaign that observes a mobile after the host ended the scan, with no candidate or narrow-phase question, is a failure of the variant on the observation alone
    let error = error_of(drive(&healthy(", pose_after_none = true"), &[("mode", "scan_none")]));
    assert!(
        error.contains("still asked 0 entity_candidate, 0 entity_collision and 1 entity_pose questions after every enumeration was empty"),
        "{error}"
    );
    // the Original profile (the fixture API and the real caller, no hosted module) asks the host nothing
    let driven = drive(&healthy(", fixture_api = true, original_tests = true"), &[("profile", "original")]);
    let row = authority(&driven);
    assert_eq!(row["pose_after"], 0);
    assert_eq!(row["pose_before"], 0);
}

#[test]
fn the_stage_masks_the_whole_identity_domain_the_prelaunch_phase_and_the_facts_are_classified_together() {
    // the placement stage reads an attitude, so an attitude difference is valid there; the facts are summed from the histograms
    let (driven, row) = pose_row(", candidate_sequence = { 'need_placement', 'test' }, pose_difference_placement = 16", "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(row["pose_histogram"], "0:4:1:recorded:1:8=1;1:4:1:recorded:1:16=1");
    for (field, expected) in [
        ("pose_fact_observations", 2),
        ("pose_fact_offers", 1),
        ("pose_fact_recorded", 1),
        ("pose_fact_no_id", 0),
        ("pose_fact_dead", 0),
        ("pose_fact_position_differs", 0),
        ("pose_fact_attitude_compared", 1),
        ("pose_fact_attitude_differs", 0),
        ("pose_fact_motion_not_held", 1),
        ("pose_fact_motion_held", 0),
        ("pose_fact_formula_equal", 1),
        ("pose_fact_vehicle_formula_equal", 1),
        ("pose_fact_vehicle_formula_differs", 0),
    ] {
        assert_eq!(row[field], expected, "{field}");
    }
    assert_eq!(
        row["pose_fact_recorded_by_kind"], "1=1",
        "per OFFER (stage 0): the placement observation of the same offer is not a second mobile"
    );
    // an aircraft record whose held vector differs: classified apart from the vehicles
    let (driven, row) = pose_row(", pose_kind = 4, pose_difference = 16", "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    for (field, expected) in [
        ("pose_fact_aircraft_motion_differs", 0),
        ("pose_fact_motion_held", 1),
        ("pose_fact_motion_held_differs", 0),
        ("pose_fact_position_differs", 0),
        ("pose_fact_formula_differs", 1),
        ("pose_fact_vehicle_formula_differs", 0),
    ] {
        assert_eq!(row[field], expected, "{field}");
    }
    // a reply the stage could not have produced is a counted reply fault and never a pass: an attitude difference at the offer, a held-vector difference with none held (both stages)
    for extra in [
        ", bad_real_pose = 'attitude_at_offer'",
        ", bad_real_pose = 'contradiction'",
        ", candidate_sequence = { 'need_placement', 'test' }, pose_difference_placement = 12",
        ", pose_difference = 2",
    ] {
        let driven = drive(&healthy(extra), &[]);
        let row = authority(&driven);
        assert!(row["reply_faults"].as_i64().unwrap() >= 1, "{extra}: {row}");
        assert!(row["first_fault"].as_str().unwrap().starts_with("pose reply:"), "{extra}: {row}");
        assert!(driven.result.is_err(), "{extra}: a malformed real reply is never a pass");
    }
    // the roster's whole identity domain is carried: 128 bytes is a valid identity, 129 is not and is counted as a request fault
    let (driven, row) = pose_row(", pose_id_length = 128", "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(driven.pose_log[0].4.len(), 15 * 10 + 14 + 1 + 128);
    assert_eq!(row["request_faults"], 0);
    let row = authority(&drive(&healthy(", pose_id_length = 129"), &[]));
    assert!(row["request_faults"].as_i64().unwrap() >= 1, "{row}");
    assert!(row["first_fault"].as_str().unwrap().starts_with("pose request:"), "{row}");
    // an observation BEFORE the launch is classified too, apart from the armed ones and never altered; reached there alone it is a reached control
    let (driven, row) = pose_row(", pose_before_launch = true", "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let pre = row["pose_pre_total"].as_i64().unwrap();
    assert!(pre >= 1, "{row}");
    assert_eq!(
        (row["pose_before"].clone(), row["pose_pre_recorded"].clone()),
        (Value::from(pre), Value::from(pre))
    );
    assert_eq!(row["pose_pre_histogram"], format!("0:4:1:recorded:1:8={pre}").as_str());
    assert_eq!(
        (row["pose_after"].clone(), row["pose_fact_offers"].clone()),
        (Value::from(1), Value::from(pre + 1)),
        "the facts sum both phases"
    );
    assert_eq!(row["pose_fact_observations"], pre + 1);
    let (driven, row) = pose_row(", pose_before_launch = true, no_pose_questions = true", "control");
    assert!(driven.result.is_ok(), "reached before the launch alone: {:?}\n{}", driven.result, driven.rows);
    assert_eq!(row["pose_after"], 0);
    assert!(row["pose_pre_total"].as_i64().unwrap() >= 1, "{row}");
    // the pre-launch observations are never altered: the refusal modes act on the armed ones only
    let row = authority(&drive(&healthy(", pose_before_launch = true"), &[("mode", "withheld_pose")]));
    assert!(row["pose_pre_total"].as_i64().unwrap() >= 1, "{row}");
    assert_eq!(
        (row["pose_altered"].clone(), row["failure_marker"].clone()),
        (Value::from(1), Value::from("entity_pose.callback"))
    );
}

#[test]
fn the_host_side_record_modes_refuse_by_exact_name_and_nothing_is_asked_afterwards() {
    // the host retires the identity of the first mobile candidate that named one: the real provider refuses it by name at the candidate question, and the narrow phase is never reached
    let driven = drive(&healthy(""), &[("mode", "withdrawn_record")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = authority(&driven);
    assert_eq!(
        (row["completed"].clone(), row["failure_marker"].clone(), row["record_withdrawn"].clone()),
        (Value::from(false), Value::from("entity_candidate.refused"), Value::from(1))
    );
    assert!(row["failure_line"].as_str().unwrap().contains("retired"), "{row}");
    assert_eq!(
        (row["candidate_after"].clone(), row["entity_after"].clone()),
        (Value::from(1), Value::from(0)),
        "the refused question is the last the host was asked"
    );
    for (field, expected) in [("case_weapons_pending", -1), ("contact_any_tests_delta", -1), ("contact_any_hits_delta", -1)] {
        assert_eq!(row[field], expected, "{field}: no counter is read from a stopped engine");
    }
    // the host places its own record elsewhere: the observation reports the difference and the engine stops at the conformance assertion, before the candidate question
    let driven = drive(&healthy(""), &[("mode", "displaced_record")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = authority(&driven);
    assert_eq!(
        (row["completed"].clone(), row["failure_marker"].clone(), row["record_displaced"].clone()),
        (Value::from(false), Value::from("entity_pose.mirror"), Value::from(1))
    );
    assert!(row["failure_line"].as_str().unwrap().contains("differ from the host"), "{row}");
    assert_eq!((row["pose_after"].clone(), row["candidate_after"].clone()), (Value::from(1), Value::from(0)));
    // a mode whose host-side action never happens (no mobile candidate named an identity) completes without its stop: a failure, never a refusal
    let error = error_of(drive(&healthy(", unnamed_candidate = true"), &[("mode", "withdrawn_record")]));
    assert!(error.contains("the engine did not stop at entity_candidate.refused"), "{error}");
    // the natural control counts the mobile candidate questions by identity, and an unnamed one is a failure of the control (the host could not decide its pose)
    let (driven, row) = scan_row("", "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(
        (row["candidate_mobile_named"].clone(), row["candidate_mobile_unnamed"].clone()),
        (Value::from(1), Value::from(0))
    );
    let error = error_of(drive(&healthy(", unnamed_candidate = true"), &[]));
    assert!(error.contains("carried no host identity"), "{error}");
    // a conformance difference in a completed-control scenario stops the engine by its name (the shared C refuses it), and is never a pass
    let driven = drive(&healthy(", pose_difference = 21"), &[]);
    let row = authority(&driven);
    assert_eq!(row["failure_marker"], "entity_pose.mirror");
    assert!(driven.result.is_err());
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the host-side movement modes: a moving routed case selected from the host's own step answers, and a terrain input the host changes after the launch

/// three mobiles: a blue launcher, a red routed vehicle the host steps at 3 m/s (the far one) and a red routed vehicle it steps with a zero vector (the near one); the campaign's own aim request of every
/// frame has the launcher engaging the far one with weapon 7 (so its link is already that target); the far one is the candidate the
/// scripted dialogue names, asks need_updated of, and tests; the cohort ends in the SECOND frame after the launch, so a lift at the end of the first is followed by a question
fn moving_scenario(extra: &str) -> String {
    healthy(&format!(
        ", mobiles = {{ {{ kind = 5, side = 'blue' }}, {{ kind = 1, side = 'red' }}, {{ kind = 1, side = 'red' }} }}, positions = {{ '0,0,0', '100,0,100', '40,0,0' }}, steps = {{ 2, 3 }}, \
         motion = {{ ['eech-world:2'] = {{ 3, 0, 0 }} }}, engagements = {{ {{ source = 1, target = 2, weapon = 7 }} }}, candidate_id = 'eech-world:2', candidate_sequence = {{ 'need_updated', 'test' }}, after = {{ [2] = {{ {TERMINAL} }} }}{extra}"
    ))
}

#[test]
fn the_moving_case_is_the_campaigns_own_engagement_of_a_routed_enemy_the_host_actually_stepped_and_its_questions_are_counted_by_identity() {
    // the natural selection is unchanged (the NEARER stationary vehicle), and the campaign's aim requests reached the host's own aim exactly as the engine made them
    let driven = drive(&moving_scenario(""), &[]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(selected(&driven)["target"], "eech-world:3");
    assert_eq!(
        selected(&driven)["selection"],
        "nearest_non_person_enemy_of_a_launcher_with_an_unguided_ballistic_weapon"
    );
    assert!(driven.vectors.0 >= 1 && !driven.vectors.1, "{:?}", driven.vectors);
    // the moving selection is the campaign's engagement the host's last step made moving, and the launch is made with THAT already-linked target and weapon
    let driven = drive(&moving_scenario(""), &[("mode", "moving_control")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = selected(&driven);
    assert_eq!(
        (row["launcher"].clone(), row["target"].clone(), row["weapon"].clone()),
        (Value::from("eech-world:1"), Value::from("eech-world:2"), Value::from(7))
    );
    assert_eq!(row["selection"], "campaign_engagement_of_a_moving_routed_enemy");
    assert_eq!(row["distance"], -1, "no distance is ranked");
    assert_eq!(driven.launch, Some(("eech-world:1".to_string(), "eech-world:2".to_string(), 7)));
    assert!(
        driven.vectors.0 >= 1 && !driven.vectors.1,
        "the aim callback forwards every request unchanged: {:?}",
        driven.vectors
    );
    let moving = row_of(&driven, "moving_target");
    assert_eq!(
        (moving["motion"].clone(), moving["frame"].clone(), moving["vectors_frame"].clone()),
        (Value::from("3,0,0"), Value::from(0), Value::from(0))
    );
    assert_eq!(moving["aim_target_motion"], "3,0,0");
    assert_eq!(moving["heading_evidence"], "unavailable_aim_step_carries_no_weapon_identity");
    // the launch used the campaign's own link: no setter, so the link before is 1 and nothing was restored
    let launch = row_of(&driven, "case_launch");
    assert!(
        launch["text"].as_str().unwrap().contains("target_link_before=1") && launch["text"].as_str().unwrap().contains("restored=-1"),
        "{launch}"
    );
    // the per-identity proof that the question was a moving one (the shared C asks need_updated only of a candidate whose vector is above 1), a test, and no lift
    let row = authority(&driven);
    assert_eq!(row["completed"], true);
    assert_eq!(
        (
            row["target"].clone(),
            row["target_need_updated"].clone(),
            row["target_test"].clone(),
            row["target_questions"].clone()
        ),
        (Value::from("eech-world:2"), Value::from(2), Value::from(2), Value::from(4)),
        "{row}"
    );
    assert_eq!(
        (row["lifted"].clone(), row["target_after_lift"].clone(), row["candidate_mobile_unnamed"].clone()),
        (Value::from(0), Value::from(0), Value::from(0))
    );
    assert_eq!(row["altered"], 0);
    assert!(driven.move_log.iter().all(|(_, _, elevation, _)| *elevation == 10.0), "{:?}", driven.move_log);
    // the lowest IDENTITY wins, not the smaller distance: two engagements of the same moving target, the launcher nearer to it has the higher identity
    let two = ", mobiles = { { kind = 5, side = 'blue' }, { kind = 1, side = 'red' }, { kind = 1, side = 'blue' } }, positions = { '0,0,0', '1000,0,1000', '998,0,1000' }, steps = { 2 }, \
               armed = { [1] = true, [3] = true }, engagements = { { source = 3, target = 2, weapon = 7 }, { source = 1, target = 2, weapon = 7 } }";
    let driven = drive(&moving_scenario(two), &[("mode", "moving_control")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(driven.launch, Some(("eech-world:1".to_string(), "eech-world:2".to_string(), 7)));
    // no moving target is engaged: the host stepped a moving enemy the campaign is NOT aiming at, and it is never taken instead
    let engaged_stationary = ", engagements = { { source = 1, target = 3, weapon = 7 } }";
    let driven = drive(&moving_scenario(engaged_stationary), &[("mode", "moving_control")]);
    let launch = driven.launch.clone();
    let error = error_of(driven);
    assert!(
        error.contains("UNREACHED: no campaign engagement of a moving routed enemy in the final warmup frame") && error.contains("not tuned"),
        "{error}"
    );
    assert_eq!(launch, None);
    // no campaign engagement at all
    let error = error_of(drive(&moving_scenario(", engagements = {}"), &[("mode", "moving_control")]));
    assert!(
        error.contains("UNREACHED: no campaign engagement of a moving routed enemy") && error.contains("(0 launchers"),
        "{error}"
    );
    // an engagement the campaign made only in an EARLIER frame than the final warmup frame does not qualify
    let error = error_of(drive(
        &moving_scenario(", engagement_until = 1"),
        &[("mode", "moving_control"), ("warmup", "3")],
    ));
    assert!(error.contains("UNREACHED: no campaign engagement of a moving routed enemy"), "{error}");
    // the selected weapon must pass the existing in-stock, unguided, non-decoy conditions: weapon 8 is not in stock
    let error = error_of(drive(
        &moving_scenario(", engagements = { { source = 1, target = 2, weapon = 8 } }"),
        &[("mode", "moving_control")],
    ));
    assert!(error.contains("UNREACHED: no campaign engagement of a moving routed enemy"), "{error}");
    // a host that stepped no routed enemy with a vector above 1 m/s, or never stepped the engaged target in the final frame, is the same blocker
    for extra in [", motion = {}", ", steps = { 3 }"] {
        let error = error_of(drive(&moving_scenario(extra), &[("mode", "moving_control")]));
        assert!(error.contains("UNREACHED: no campaign engagement of a moving routed enemy"), "{extra}: {error}");
    }
    // a launch that had to SET the link was not the campaign's own engagement: a failure, never a pass
    let wrong_report = report("target_link_before=0");
    let error = error_of(drive(
        &moving_scenario(&format!(", launch_report = \"{wrong_report}\"")),
        &[("mode", "moving_control")],
    ));
    assert!(error.contains("the launch did not use the campaign's own engagement"), "{error}");
    // a moving selection whose target is never asked need_updated (the dialogue never saw a vector above 1) is UNREACHED, never a pass
    let error = error_of(drive(&moving_scenario(", candidate_sequence = { 'test' }"), &[("mode", "moving_control")]));
    assert!(
        error.contains("UNREACHED: moving_control") && error.contains("never asked need_updated"),
        "{error}"
    );
    // a target no question reaches the narrow phase of is UNREACHED as well
    let error = error_of(drive(
        &moving_scenario(", candidate_sequence = { 'need_updated', 'skip' }"),
        &[("mode", "moving_control")],
    ));
    assert!(
        error.contains("UNREACHED") && error.contains("no candidate reached the narrow phase"),
        "{error}"
    );
}

#[test]
fn a_terrain_lift_changes_only_the_targets_own_routed_step_after_the_launch_and_needs_a_later_question_about_it() {
    let driven = drive(&moving_scenario(""), &[("mode", "terrain_lift")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = authority(&driven);
    assert_eq!(row["completed"], true);
    // two armed steps of the target, each lifted by exactly the declared constant, answered AT the lifted elevation; the later question about the target saw a lift and was tested
    assert_eq!((row["lifted"].clone(), row["lift_refused"].clone()), (Value::from(2), Value::from(0)), "{row}");
    for (field, expected) in [
        ("lift_constant", 500.0),
        ("lift_baseline_elevation", 10.0),
        ("lift_elevation", 510.0),
        ("lift_answered_y", 510.0),
    ] {
        assert_eq!(row[field].as_f64(), Some(expected), "{field}: {row}");
    }
    assert!(
        row["target_after_lift"].as_i64().unwrap() >= 1 && row["target_after_lift_test"].as_i64().unwrap() >= 1,
        "{row}"
    );
    assert!(row["target_need_updated"].as_i64().unwrap() >= 1);
    // the host saw: every warmup step and the other mobile's steps unchanged, the target's armed steps lifted, nothing else touched; the engine's own table was never mutated
    let target = "eech-world:2".to_string();
    for (id, mode, elevation, launched) in &driven.move_log {
        assert_eq!(mode, "move");
        let lifted = *launched && *id == target;
        assert_eq!(*elevation, if lifted { 510.0 } else { 10.0 }, "{id} launched={launched}");
    }
    assert!(driven.move_log.iter().any(|(id, _, _, launched)| *launched && *id == "eech-world:3"));
    assert!(!driven.caller_mutated, "the engine's step table was mutated by the wrapper");
    // the unaltered control of the same scenario lifts nothing (the effect is the lift, not the case)
    let control = drive(&moving_scenario(""), &[("mode", "moving_control")]);
    assert!(control.move_log.iter().all(|(_, _, elevation, _)| *elevation == 10.0));
    // a lift the step's own map ceiling does not allow is not made, and the run is UNREACHED
    let error = error_of(drive(&moving_scenario(", ceiling = 100"), &[("mode", "terrain_lift")]));
    assert!(error.contains("UNREACHED: terrain_lift lifted no step"), "{error}");
    // a cohort that ends before any question follows the lifted step is UNREACHED (weapon first, vehicle after: the first question that can see a lift is in the next frame)
    let error = error_of(drive(
        &moving_scenario(&format!(", after = {{ [1] = {{ {TERMINAL} }} }}")),
        &[("mode", "terrain_lift")],
    ));
    assert!(error.contains("UNREACHED: terrain_lift: the cohort ended before a lifted step"), "{error}");
    // a host whose answer is not at the lifted elevation did not take the input: a failure, never a pass
    let error = error_of(drive(&moving_scenario(", answer_ignores_lift = true"), &[("mode", "terrain_lift")]));
    assert!(error.contains("not at the lifted elevation"), "{error}");
    // the lift needs the moving selection's target: with none moving the run is the same exact blocker
    let error = error_of(drive(&moving_scenario(", motion = {}"), &[("mode", "terrain_lift")]));
    assert!(error.contains("UNREACHED: no campaign engagement of a moving routed enemy"), "{error}");
}

#[test]
fn the_host_side_movement_modes_run_under_the_original_profile_and_the_refusal_modes_still_do_not() {
    let original = [("profile", "original"), ("mode", "terrain_lift")];
    // the original asks the host no entity question, so the run ends in the expected capability red, after the lift was verified; its counters say what was and was not compared
    let driven = drive(&moving_scenario(", fixture_api = true, original_tests = true"), &original);
    let rows_text = driven.rows.clone();
    let row = authority(&driven);
    assert_eq!(
        (row["lifted"].clone(), row["target"].clone(), row["candidate_after"].clone()),
        (Value::from(2), Value::from("eech-world:2"), Value::from(0)),
        "{rows_text}"
    );
    assert!(error_of(driven).contains("CAPABILITY RED"));
    // no lift, no original comparison: the same exact blocker as the hosted one
    let error = error_of(drive(&moving_scenario(", fixture_api = true, original_tests = true, ceiling = 100"), &original));
    assert!(error.contains("UNREACHED: terrain_lift lifted no step"), "{error}");
    for mode in ["withdrawn_record", "displaced_record", "forced_hit"] {
        assert!(
            error_of(drive("", &[("profile", "original"), ("mode", mode)])).contains("only mode=control"),
            "{mode}"
        );
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the controlled moving QUERY case: three latched segments about one moving routed vehicle, one actual frame (with the lift, or not), then the SAME segments asked of the production query

/// the moving scenario of the engagement tests, with the query surface: the host steps the far vehicle at 3 m/s and the near one with a zero vector; the scripted production query's dialogue names the
/// far one, which is asked need_updated and then tested with the 16-word moving pose. No weapon is launched in these modes.
fn query_scenario(extra: &str) -> String {
    moving_scenario(&format!(", moving = true{extra}"))
}

fn query_row(driven: &Driven) -> Value {
    row_of(driven, "query_authority")
}

#[test]
fn the_controlled_query_latches_three_segments_about_the_first_moving_routed_vehicle_runs_one_frame_and_asks_the_same_segments() {
    let driven = drive(&query_scenario(""), &[("mode", "moving_query_control")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    // the target is the host-stepped moving vehicle in identity order (never the nearer stationary one); no weapon was launched or engaged
    let selected = row_of(&driven, "query_case_selected");
    assert_eq!(
        (selected["target"].clone(), selected["selection"].clone()),
        (Value::from("eech-world:2"), Value::from("first_alive_moving_routed_target_in_identity_order"))
    );
    assert_eq!(driven.launch, None);
    assert!(
        driven
            .rows
            .lines()
            .all(|line| !line.contains("\"event\":\"case_launch\"") && !line.contains("\"event\":\"cohort\"")),
        "{}",
        driven.rows
    );
    // prepare, then ONE actual frame (one second of the scripted clock), then run: the latch is made before the frame and consumed after it
    let times: Vec<f64> = driven
        .query_calls
        .iter()
        .map(|call| call.split('@').nth(1).unwrap().split(':').next().unwrap().parse().unwrap())
        .collect();
    assert_eq!(
        driven.query_calls.iter().map(|call| call.split('@').next().unwrap()).collect::<Vec<_>>(),
        ["prepare", "run"]
    );
    assert_eq!(times[1] - times[0], 1.0, "{:?}", driven.query_calls);
    let prepared = row_of(&driven, "query_prepared");
    assert_eq!(
        (prepared["target"].clone(), prepared["target_type"].clone()),
        (Value::from("eech-world:2"), Value::from("ENTITY_TYPE_ROUTED_VEHICLE"))
    );
    let row = query_row(&driven);
    assert_eq!(row["completed"], true);
    assert_eq!(row["segments"], 3);
    // the physical outputs of the three segments, reported by the production caller: a hit carries its point and normal, a miss none
    let segments: Vec<Value> = rows(&driven).into_iter().filter(|r| r["event"] == "query_segment").collect();
    assert_eq!(segments.len(), 3);
    assert_eq!(
        (segments[0]["hit"].clone(), segments[1]["hit"].clone(), segments[2]["hit"].clone()),
        (Value::from(1), Value::from(0), Value::from(1))
    );
    assert_eq!(
        (segments[1]["point"].clone(), segments[1]["normal"].clone()),
        (Value::from("-"), Value::from("-"))
    );
    assert_eq!(row["query_hits"], 2);
    // the target-specific coverage the case requires, counted by identity: every one of the three scans asked the target need_updated, then tested it with the 16-word moving pose
    assert_eq!(
        (
            row["target_need_updated"].clone(),
            row["target_test"].clone(),
            row["target_test_moving16"].clone(),
            row["candidate_mobile_unnamed"].clone()
        ),
        (Value::from(3), Value::from(3), Value::from(3), Value::from(0)),
        "{row}"
    );
    assert_eq!(
        (row["lifted"].clone(), row["natural_weapon_engagement"].clone()),
        (Value::from(0), Value::from("not_claimed_query_service_only"))
    );
    assert!(driven.move_log.iter().all(|(_, _, elevation, _)| *elevation == 10.0), "{:?}", driven.move_log);
    assert!(!driven.caller_mutated);

    // the changed mode lifts ONLY the target's own host step of that one frame; the latched segments are the same; the target's questions followed the lifted step
    let control_signature = row["query_signature"].as_str().unwrap().to_string();
    let lifted = drive(
        &query_scenario(", query_hits = { 1, 0, 0 }"),
        &[("mode", "moving_query_lift"), ("control_signature", &control_signature)],
    );
    assert!(lifted.result.is_ok(), "{:?}\n{}", lifted.result, lifted.rows);
    let changed = query_row(&lifted);
    assert_eq!(changed["completed"], true);
    assert_eq!(changed["lifted"], 1);
    for (field, expected) in [("lift_baseline_elevation", 10.0), ("lift_elevation", 510.0), ("lift_answered_y", 510.0)] {
        assert_eq!(changed[field].as_f64(), Some(expected), "{field}: {changed}");
    }
    assert!(
        changed["target_after_lift"].as_i64().unwrap() >= 1 && changed["target_after_lift_test"].as_i64().unwrap() >= 1,
        "{changed}"
    );
    assert_eq!(
        row_of(&lifted, "query_prepared")["segments"],
        prepared["segments"],
        "the latched segments are the same in both modes"
    );
    for (id, _, elevation, launched) in &lifted.move_log {
        assert!(!launched);
        let in_the_frame = *id == "eech-world:2" && *elevation == 510.0;
        assert!(in_the_frame || *elevation == 10.0, "{id} {elevation}");
    }
    assert_eq!(
        lifted.move_log.iter().filter(|(_, _, elevation, _)| *elevation == 510.0).count(),
        1,
        "exactly the one frame's step of the target: {:?}",
        lifted.move_log
    );
    assert!(!lifted.caller_mutated);
}

#[test]
fn a_query_case_that_cannot_be_prepared_or_covered_is_an_exact_unreached_and_never_a_pass() {
    let control = [("mode", "moving_query_control")];
    // no routed vehicle was stepped above 1 m/s in the final warmup frame: nothing is searched, retried or tuned
    let error = error_of(drive(&query_scenario(", motion = {}"), &control));
    assert!(
        error.contains("UNREACHED: no alive routed vehicle was stepped by the host above 1 m/s") && error.contains("nothing is tuned"),
        "{error}"
    );
    // the engine refuses the latch (an unsupported model, target or non-finite input): named, and the query is never asked
    let driven = drive(&query_scenario(", query_refuse = 'prepare'"), &control);
    let calls = driven.query_calls.clone();
    let error = error_of(driven);
    assert!(
        error.contains("UNREACHED: the controlled query case refused wpn_moving_query_prepare") && error.contains("target_model_unsupported"),
        "{error}"
    );
    assert_eq!(calls.len(), 1);
    // the engine refuses the run (the identity no longer names an alive routed vehicle)
    let error = error_of(drive(&query_scenario(", query_refuse = 'run'"), &control));
    assert!(
        error.contains("UNREACHED: the controlled query case refused wpn_moving_query_run") && error.contains("target_not_alive"),
        "{error}"
    );
    // the case's named refusal reaches the driver in the form the REAL bridge delivers it; a failure that lost its reason is not a refusal of the case
    let error = error_of(drive(&query_scenario(", query_refuse = 'prepare_lost'"), &control));
    assert!(error.contains("without a named case refusal"), "{error}");
    // the segments asked must be the ones latched before the frame
    let error = error_of(drive(&query_scenario(", query_segment_changed = true"), &control));
    assert!(error.contains("malformed") && error.contains("is not the prepared segment"), "{error}");
    // a module without the surface is not the weapon fixture
    let error = error_of(drive(&query_scenario(", no_query_surface = true"), &control));
    assert!(error.contains("no controlled moving query surface"), "{error}");
    // coverage: the target must be asked need_updated, reach a test and the test must carry the 16-word moving pose
    let error = error_of(drive(&query_scenario(", candidate_sequence = { 'test' }"), &control));
    assert!(
        error.contains("UNREACHED: moving_query_control") && error.contains("never asked need_updated"),
        "{error}"
    );
    let error = error_of(drive(&query_scenario(", candidate_sequence = { 'need_updated', 'skip' }"), &control));
    assert!(error.contains("UNREACHED") && error.contains("reached the narrow phase"), "{error}");
    let error = error_of(drive(&query_scenario(", moving = false"), &control));
    assert!(
        error.contains("UNREACHED: moving_query_control") && error.contains("16-word moving pose"),
        "{error}"
    );
    // an identity-less mobile question is a failure of the control, not a pass
    let error = error_of(drive(&query_scenario(", unnamed_candidate = true"), &control));
    assert!(error.contains("carried no host identity"), "{error}");
    // the lift is bounded by the step's own map ceiling and a refused lift is UNREACHED
    let lift = [("mode", "moving_query_lift"), ("control_signature", "x")];
    let error = error_of(drive(&query_scenario(", ceiling = 100"), &lift));
    assert!(error.contains("UNREACHED: moving_query_lift lifted no step"), "{error}");
    // a host answer that does not take the lifted input is a failure
    let error = error_of(drive(&query_scenario(", answer_ignores_lift = true"), &lift));
    assert!(error.contains("not at the lifted elevation"), "{error}");
    // a conformance difference stops the engine and is never a pass
    let driven = drive(&query_scenario(", pose_difference = 21"), &control);
    assert!(driven.result.is_err(), "{}", driven.rows);
    assert_eq!(query_row(&driven)["failure_marker"], "entity_pose.mirror");
}

/// The roster identity of 128 bytes the scripted collaborator names, and its encoding computed here independently of the driver and of the scripted encoder (the C codec's rule: ASCII letters, digits and
/// `._~` stay, every other byte is `%XX` in upper-case hexadecimal).
fn exotic_identity() -> Vec<u8> {
    let cycle: [u8; 18] = [32, 10, 97, 9, 37, 195, 169, 45, 61, 13, 90, 226, 130, 172, 126, 46, 95, 57];
    (0..128).map(|k| cycle[k % cycle.len()]).collect()
}

fn encoded_identity(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'~') {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

#[test]
fn a_complete_c_shaped_report_with_a_maximum_whitespace_and_utf8_identity_and_a_building_hit_is_accepted_and_reported_exactly() {
    let encoded = encoded_identity(&exotic_identity());
    let driven = drive(&query_scenario(", query_other_hit_exotic = true"), &[("mode", "moving_query_control")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let segments: Vec<Value> = rows(&driven).into_iter().filter(|r| r["event"] == "query_segment").collect();
    assert_eq!(segments.len(), 3);
    // the other entity's identity is carried canonically encoded (no whitespace, exactly the independently encoded bytes), its type is the compiled name, and the hit's words are kept as printed
    assert_eq!(segments[0]["hit_host_id"], Value::from(encoded.clone()));
    assert!(!encoded.chars().any(|c| c.is_whitespace()) && encoded.len() > 128, "{encoded}");
    assert_eq!(segments[0]["hit_type"], "ENTITY_TYPE_HELICOPTER");
    assert_eq!(segments[0]["hit_is_target"], 0);
    assert_eq!(segments[0]["point"], "0xffffffff,0x7fc00001,0x80000000");
    assert!(query_row(&driven)["query_signature"].as_str().unwrap().contains(&encoded));
    // a hit entity with no roster identity carries `-`, which is never an identity
    let driven = drive(&query_scenario(", query_building_hit = true"), &[("mode", "moving_query_control")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let segments: Vec<Value> = rows(&driven).into_iter().filter(|r| r["event"] == "query_segment").collect();
    assert_eq!(
        (segments[0]["hit_type"].clone(), segments[0]["hit_host_id"].clone()),
        (Value::from("ENTITY_TYPE_CITY_BUILDING"), Value::from("-"))
    );
}

#[test]
fn every_single_fault_in_a_prepare_or_run_report_stops_the_witness_and_a_complete_report_does_not() {
    let control = [("mode", "moving_query_control")];
    // positive first: the complete valid report, whose first hit is the target itself (flag, identity and type agree), is accepted; every fault below corrupts that one report
    let driven = drive(&query_scenario(""), &control);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let first = rows(&driven).into_iter().find(|r| r["event"] == "query_segment").unwrap();
    assert_eq!(
        (first["hit_is_target"].clone(), first["hit_type"].clone()),
        (Value::from(1), Value::from("ENTITY_TYPE_ROUTED_VEHICLE")),
        "{first}"
    );
    // (the scripted single fault, the part of the witness failure that names it)
    let faults: [(&str, &str); 29] = [
        ("prepare_unknown_record", "prepare report has 5 records"),
        ("prepare_missing_segment", "prepare report has 3 records"),
        ("prepare_duplicate_segment", "out of order"),
        ("prepare_other_target", "does not name the target"),
        ("prepare_bad_bearing", "undeclared bearing"),
        ("prepare_nonfinite_endpoint", "non-finite endpoint"),
        ("prepare_wrong_type", "segment count or target type"),
        ("prepare_double_space", "not single-spaced"),
        ("prepare_extra_key", "with 4 keys"),
        ("prepare_no_newline", "newline-terminated"),
        ("run_extra_row", "run report has 5 records"),
        ("run_missing_row", "run report has 3 records"),
        ("run_other_target", "does not name the target"),
        ("run_hit_without_point", "three point words and three normal words"),
        ("run_hit_two_normal_words", "three point words and three normal words"),
        ("run_miss_with_point", "miss that carries hit fields"),
        ("run_miss_with_identity", "miss that carries hit fields"),
        ("run_target_flag_with_other_identity", "identity or type is not"),
        ("run_target_identity_with_false_flag", "hit is not the target, but its identity is the target's"),
        (
            "run_target_identity_with_false_flag_and_other_type",
            "hit is not the target, but its identity is the target's",
        ),
        ("run_bad_encoding", "not a canonical encoding"),
        ("run_lowercase_escape", "not a canonical encoding"),
        ("run_non_canonical_escape", "not a canonical encoding"),
        ("run_short_word", "three point words and three normal words"),
        ("run_unknown_key", "with 9 keys"),
        ("run_reordered_keys", "unknown or out of order"),
        ("run_bad_hit_flag", "carries hit=2"),
        ("run_hit_is_target_unknown", "no hit_is_target flag"),
        ("run_trailing_space", "not single-spaced"),
    ];
    for (fault, expected) in faults {
        let driven = drive(&query_scenario(&format!(", query_fault = '{fault}'")), &control);
        let rows_text = driven.rows.clone();
        let error = error_of(driven);
        assert!(error.contains("malformed") && error.contains(expected), "{fault}: {error}");
        // no completed result is ever published for a malformed report
        assert!(
            !rows_text.contains("\"event\":\"query_authority\"") || !rows_text.contains("\"completed\":true"),
            "{fault}: {rows_text}"
        );
    }
}

#[test]
fn the_lifted_query_is_service_evidence_only_with_a_post_lift_offer_and_a_changed_physical_result_and_may_skip_the_narrow_phase() {
    let lift = |signature: &'static str| [("mode", "moving_query_lift"), ("control_signature", signature)];
    // the control's physical result, as the control run would report it
    let control = drive(&query_scenario(""), &[("mode", "moving_query_control")]);
    let signature = query_row(&control)["query_signature"].as_str().unwrap().to_string();
    let signature: &'static str = Box::leak(signature.into_boxed_str());
    // the lifted target is offered after the lift and skipped by the broad phase (no need_updated, no test, no 16-word pose), and the physical result changed: accepted
    let driven = drive(&query_scenario(", candidate_sequence = { 'skip' }, query_hits = { 0, 0, 0 }"), &lift(signature));
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = query_row(&driven);
    assert_eq!(
        (
            row["target_after_lift"].clone(),
            row["target_need_updated"].clone(),
            row["target_test"].clone(),
            row["target_test_moving16"].clone(),
            row["query_hits"].clone()
        ),
        (Value::from(3), Value::from(0), Value::from(0), Value::from(0), Value::from(0)),
        "{row}"
    );
    // the same lifted run whose physical result equals the control's is not changed-pose authority
    let error = error_of(drive(&query_scenario(""), &lift(signature)));
    assert!(error.contains("UNCHANGED") && error.contains("changed no output"), "{error}");
    // a changed result with no control signature supplied is not compared, so it is not closed
    let error = error_of(drive(&query_scenario(", query_hits = { 0, 0, 0 }"), &[("mode", "moving_query_lift")]));
    assert!(error.contains("needs control_signature="), "{error}");
    // no question about the target after the lift is UNREACHED, whatever the pre-phase counts were (the unlifted control still needs its strict moving path)
    let error = error_of(drive(
        &query_scenario(", query_hits = { 0, 0, 0 }, candidate_id = 'eech-world:3'"),
        &lift(signature),
    ));
    assert!(error.contains("UNREACHED") && error.contains("followed the lifted step"), "{error}");
    // the control (A) keeps its strict moving predicates even where B may skip
    let error = error_of(drive(&query_scenario(", candidate_sequence = { 'skip' }"), &[("mode", "moving_query_control")]));
    assert!(
        error.contains("UNREACHED: moving_query_control") && error.contains("never asked need_updated"),
        "{error}"
    );
}

#[test]
fn the_controlled_query_runs_the_same_case_under_the_original_profile_and_reports_outputs_only() {
    let original = |mode: &'static str| [("profile", "original"), ("mode", mode)];
    for mode in ["moving_query_control", "moving_query_lift"] {
        let driven = drive(&query_scenario(", fixture_api = true, query_original = true"), &original(mode));
        assert!(driven.result.is_ok(), "{mode}: {:?}\n{}", driven.result, driven.rows);
        let row = query_row(&driven);
        assert_eq!(
            (
                row["completed"].clone(),
                row["segments"].clone(),
                row["candidate_after"].clone(),
                row["entity_after"].clone(),
                row["scan_after"].clone()
            ),
            (Value::from(true), Value::from(3), Value::from(0), Value::from(0), Value::from(0)),
            "{mode}"
        );
        assert_eq!(row["target"], "eech-world:2");
    }
    // the lift is made under the original profile too (it is a movement input, not an entity answer)
    let driven = drive(&query_scenario(", fixture_api = true, query_original = true"), &original("moving_query_lift"));
    assert_eq!(query_row(&driven)["lifted"], 1);
    // a module that asks the host under the original profile is not the original
    let error = error_of(drive(&query_scenario(", fixture_api = true"), &original("moving_query_control")));
    assert!(error.contains("the module is not the original"), "{error}");
    // the hosted profile still refuses the original's fixture API
    let error = error_of(drive(&query_scenario(", fixture_api = true"), &[("mode", "moving_query_control")]));
    assert!(error.contains("keeps the original narrow phase"), "{error}");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the controlled query's callback eligibility: the actor must be one whose final warmup `move` step the host was given WITH the terrain input the bounded lift reads

/// both red vehicles are stepped at 3 m/s, so the first in identity order (2) is the moving criterion's choice unless its own step lacks the lift input; the scripted dialogue names the one in `extra`
fn two_movers(extra: &str) -> String {
    query_scenario(&format!(
        ", motion = {{ ['eech-world:2'] = {{ 3, 0, 0 }}, ['eech-world:3'] = {{ 3, 0, 0 }} }}{extra}"
    ))
}

fn steps_of(driven: &Driven, id: &str) -> Vec<(f64, bool)> {
    driven
        .move_log
        .iter()
        .filter(|(step, mode, ..)| step == id && mode == "move")
        .map(|(_, _, elevation, launched)| (*elevation, *launched))
        .collect()
}

#[test]
fn the_query_target_is_the_first_moving_vehicle_whose_final_warmup_step_carried_the_lift_input_and_an_earlier_one_without_it_is_passed_over_unaltered() {
    let control = [("mode", "moving_query_control")];
    // positive first: both movers carry the input, so the moving criterion's first (2) is the target and both are counted eligible
    let driven = drive(&two_movers(""), &control);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let selected = row_of(&driven, "query_case_selected");
    assert_eq!(
        (
            selected["target"].clone(),
            selected["moving_candidates"].clone(),
            selected["lift_input_candidates"].clone(),
            selected["eligibility"].clone()
        ),
        (
            Value::from("eech-world:2"),
            Value::from(2),
            Value::from(2),
            Value::from("final_warmup_move_step_carried_terrain_input_and_map_ceiling")
        ),
        "{selected}"
    );
    // the earlier mover's own step carries no terrain (the supported None input): it still moves, so it is counted moving, but it cannot carry the lift and the later one is the target
    let bare = two_movers(", bare_steps = { [2] = true }, candidate_id = 'eech-world:3'");
    let driven = drive(&bare, &control);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let selected = row_of(&driven, "query_case_selected");
    assert_eq!(
        (
            selected["target"].clone(),
            selected["moving_candidates"].clone(),
            selected["lift_input_candidates"].clone()
        ),
        (Value::from("eech-world:3"), Value::from(2), Value::from(1)),
        "{selected}"
    );
    assert_eq!(row_of(&driven, "query_prepared")["target"], "eech-world:3");
    // nothing was given a terrain input and no step was dropped, repeated or mutated: the passed-over mover's steps are all terrain-less, exactly as many as the target's, each delegated once
    assert!(!driven.caller_mutated);
    let passed_over = steps_of(&driven, "eech-world:2");
    assert!(
        !passed_over.is_empty() && passed_over.iter().all(|(elevation, _)| *elevation == -1.0),
        "{passed_over:?}"
    );
    assert_eq!(passed_over.len(), steps_of(&driven, "eech-world:3").len());
    // the same eligibility selects in the lifted mode (the control's physical result is the comparison) and the lift lands on that target's own real step only
    let signature = query_row(&driven)["query_signature"].as_str().unwrap().to_string();
    let signature: &'static str = Box::leak(signature.into_boxed_str());
    let lifted_scenario = format!("{bare}, candidate_sequence = {{ 'skip' }}, query_hits = {{ 0, 0, 0 }}");
    let lift = [("mode", "moving_query_lift"), ("control_signature", signature)];
    let driven = drive(&lifted_scenario, &lift);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(row_of(&driven, "query_prepared")["target"], "eech-world:3");
    let row = query_row(&driven);
    assert_eq!(
        (
            row["lifted"].clone(),
            row["window_target_steps"].clone(),
            row["window_target_moves"].clone(),
            row["window_target_moves_with_terrain"].clone()
        ),
        (Value::from(1), Value::from(1), Value::from(1), Value::from(1)),
        "{row}"
    );
    assert!(row["lift_frame"].as_i64().unwrap() >= 0, "{row}");
    assert_eq!(steps_of(&driven, "eech-world:3").iter().filter(|(elevation, _)| *elevation == 510.0).count(), 1);
    assert!(steps_of(&driven, "eech-world:2").iter().all(|(elevation, _)| *elevation == -1.0));
    // and under the original profile, which asks the host nothing but is given the same steps: the same actor
    let original = [("profile", "original"), ("mode", "moving_query_control")];
    let driven = drive(&format!("{bare}, fixture_api = true, query_original = true"), &original);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(row_of(&driven, "query_prepared")["target"], "eech-world:3");
}

#[test]
fn no_eligible_actor_is_a_named_prelaunch_unreached_with_finite_counts_and_a_terrain_less_ordinary_step_stays_valid() {
    let both_bare = ", bare_steps = { [2] = true, [3] = true }";
    for mode in ["moving_query_control", "moving_query_lift"] {
        let driven = drive(&two_movers(both_bare), &[("mode", mode), ("control_signature", "x")]);
        // the case stopped before anything was latched or run, and the counts are in the row and the reason
        assert!(driven.query_calls.is_empty(), "{mode}: {:?}", driven.query_calls);
        let selected = row_of(&driven, "query_case_selected");
        assert_eq!(
            (
                selected["target"].clone(),
                selected["moving_candidates"].clone(),
                selected["lift_input_candidates"].clone()
            ),
            (Value::from("none"), Value::from(2), Value::from(0)),
            "{mode}: {selected}"
        );
        let error = error_of(driven);
        assert!(
            error.contains("UNREACHED: 2 alive routed vehicles") && error.contains("none of those move steps carried the terrain input and map ceiling"),
            "{mode}: {error}"
        );
    }
    // no mover at all keeps the earlier, different reason
    let error = error_of(drive(&query_scenario(", motion = {}"), &[("mode", "moving_query_control")]));
    assert!(
        error.contains("UNREACHED: no alive routed vehicle was stepped by the host above 1 m/s"),
        "{error}"
    );
    // a terrain-less step is valid ordinary movement: the natural moving case selects the same vehicle and completes with the host stepping both vehicles without terrain
    let driven = drive(&moving_scenario(both_bare), &[("mode", "moving_control")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert!(!driven.caller_mutated);
    assert!(steps_of(&driven, "eech-world:2").iter().all(|(elevation, _)| *elevation == -1.0));
    assert_eq!(row_of(&driven, "case_selected")["target"], "eech-world:2");
}

#[test]
fn a_lift_is_credited_only_by_the_targets_real_terrain_bearing_step_inside_the_query_frame() {
    let lift = |signature: &'static str| [("mode", "moving_query_lift"), ("control_signature", signature)];
    let control = drive(&query_scenario(""), &[("mode", "moving_query_control")]);
    // the control reports the same window provenance (it lifts nothing)
    assert_eq!(
        (
            query_row(&control)["lifted"].clone(),
            query_row(&control)["window_target_moves_with_terrain"].clone()
        ),
        (Value::from(0), Value::from(1))
    );
    let signature = query_row(&control)["query_signature"].as_str().unwrap().to_string();
    let signature: &'static str = Box::leak(signature.into_boxed_str());
    // the actor was eligible at the final warmup frame, but its step in the query frame carries no terrain: nothing is lifted or credited, and the reason carries the finite window counts
    let scenario = query_scenario(", bare_in_query_frame = { [2] = true }, candidate_sequence = { 'skip' }, query_hits = { 0, 0, 0 }");
    let driven = drive(&scenario, &lift(signature));
    let row = query_row(&driven);
    assert_eq!(
        (
            row["lifted"].clone(),
            row["lift_refused"].clone(),
            row["lift_frame"].clone(),
            row["target_after_lift"].clone(),
            row["window_target_steps"].clone(),
            row["window_target_moves"].clone(),
            row["window_target_moves_with_terrain"].clone()
        ),
        (
            Value::from(0),
            Value::from(0),
            Value::from(-1),
            Value::from(0),
            Value::from(1),
            Value::from(1),
            Value::from(0)
        ),
        "{row}"
    );
    assert!(steps_of(&driven, "eech-world:2").iter().all(|(elevation, _)| *elevation != 510.0));
    let error = error_of(driven);
    assert!(
        error.contains("UNREACHED: moving_query_lift lifted no step")
            && error.contains("the target was given 1 steps, 1 of mode move, 0 of those with a terrain input"),
        "{error}"
    );
    // the same run with the real terrain-bearing step in the window is credited
    let driven = drive(&query_scenario(", candidate_sequence = { 'skip' }, query_hits = { 0, 0, 0 }"), &lift(signature));
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(query_row(&driven)["lifted"], 1);
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the controlled query with a configured SPEED-DEMAND input: the target is chosen by the finite positive desired_velocity its final warmup move step carried (no terrain is required), and in B only that
// target's one query-frame move step is delegated with desired_velocity 0 in place of the engine's demand. A changed demand the solver receives, not a claim about an unchanged command.

const SPEED_CONTROL: &str = "moving_query_speed_control";
const SPEED_ZERO: &str = "moving_query_speed_zero";

/// both red vehicles carry a speed demand in every step (2 asks for 7.5 and 3 for 6); the rays answer all-miss only for a target whose solver was last given a zero demand
fn demand_movers(extra: &str) -> String {
    two_movers(&format!(", demand = {{ [2] = 7.5, [3] = 6 }}, query_zero_demand_hits = {{ 0, 0, 0 }}{extra}"))
}

/// the move steps the host received for one identity in one phase
fn demand_steps(driven: &Driven, id: &str, phase: &str) -> Vec<Vec<String>> {
    driven
        .demand_log
        .iter()
        .map(|entry| entry.split('|').map(str::to_string).collect::<Vec<_>>())
        .filter(|fields| fields[0] == id && fields[2] == phase)
        .collect()
}

fn signature_of(driven: &Driven) -> &'static str {
    let text = query_row(driven)["query_signature"].as_str().expect("the signature").to_string();
    Box::leak(text.into_boxed_str())
}

#[test]
fn the_speed_control_selects_the_first_moving_vehicle_that_carried_a_positive_demand_and_changes_nothing() {
    let control = [("mode", SPEED_CONTROL)];
    let driven = drive(&demand_movers(""), &control);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let selected = row_of(&driven, "query_case_selected");
    assert_eq!(
        (
            selected["target"].clone(),
            selected["moving_candidates"].clone(),
            selected["speed_input_candidates"].clone(),
            selected["selected_demand"].clone(),
            selected["eligibility"].clone()
        ),
        (
            Value::from("eech-world:2"),
            Value::from(2),
            Value::from(2),
            Value::from("7.5"),
            Value::from("final_warmup_move_step_carried_finite_positive_desired_velocity")
        ),
        "{selected}"
    );
    // A keeps its strict moving path in the query phase, takes the window provenance, and replaces nothing: every step reached the host as the very engine table
    let row = query_row(&driven);
    assert!(
        row["target_need_updated"].as_i64().unwrap() >= 1 && row["target_test"].as_i64().unwrap() >= 1 && row["target_test_moving16"].as_i64().unwrap() >= 1,
        "{row}"
    );
    assert_eq!(
        (
            row["speed_window_valid"].clone(),
            row["speed_window_bad"].clone(),
            row["speed_transforms"].clone(),
            row["speed_delegations"].clone(),
            row["speed_incoming_demand"].clone(),
            row["speed_substituted_demand"].clone()
        ),
        (
            Value::from(1),
            Value::from(0),
            Value::from(0),
            Value::from(0),
            Value::from("none"),
            Value::from("none")
        ),
        "{row}"
    );
    assert!(!driven.caller_mutated);
    assert!(
        driven.demand_log.iter().all(|entry| entry.split('|').nth(3) == Some("-")),
        "{:?}",
        driven.demand_log
    );
    // the terrain input is not part of this selection: the same actor, with steps that carry none, is still the target
    let bare = demand_movers(", bare_steps = { [2] = true, [3] = true }");
    let driven = drive(&bare, &control);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(row_of(&driven, "query_prepared")["target"], "eech-world:2");
    assert_eq!(query_row(&driven)["window_target_moves_with_terrain"], 0);
    // the old terrain-input selector is unchanged beside it: those steps are all terrain-less, so the lift-input mode finds nothing
    let error = error_of(drive(&bare, &[("mode", "moving_query_control")]));
    assert!(error.contains("none of those move steps carried the terrain input and map ceiling"), "{error}");
}

#[test]
fn an_earlier_mover_without_a_finite_positive_demand_is_passed_over_untouched() {
    for earlier in ["0", "-2", "0/0", "1/0", "-1/0", "'fast'"] {
        let scenario = two_movers(&format!(", demand = {{ [2] = {earlier}, [3] = 6 }}, candidate_id = 'eech-world:3'"));
        let driven = drive(&scenario, &[("mode", SPEED_CONTROL)]);
        assert!(driven.result.is_ok(), "{earlier}: {:?}\n{}", driven.result, driven.rows);
        let selected = row_of(&driven, "query_case_selected");
        assert_eq!(
            (
                selected["target"].clone(),
                selected["moving_candidates"].clone(),
                selected["speed_input_candidates"].clone()
            ),
            (Value::from("eech-world:3"), Value::from(2), Value::from(1)),
            "{earlier}: {selected}"
        );
        assert!(!driven.caller_mutated, "{earlier}");
        assert!(
            driven.demand_log.iter().all(|entry| entry.split('|').nth(3) == Some("-")),
            "{earlier}: {:?}",
            driven.demand_log
        );
    }
    // a step with no desired_velocity at all
    let driven = drive(&two_movers(", demand = { [3] = 6 }, candidate_id = 'eech-world:3'"), &[("mode", SPEED_CONTROL)]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(row_of(&driven, "query_prepared")["target"], "eech-world:3");
    assert_eq!(row_of(&driven, "query_case_selected")["speed_input_candidates"], 1);
}

#[test]
fn no_actor_with_a_positive_demand_is_a_named_prelaunch_unreached_in_every_speed_mode_and_profile() {
    for demand in ["", ", demand = { [2] = 0, [3] = -1 }", ", demand = { [2] = 0/0, [3] = 1/0 }"] {
        for (mode, profile) in [
            (SPEED_CONTROL, "hosted"),
            (SPEED_ZERO, "hosted"),
            (SPEED_CONTROL, "original"),
            (SPEED_ZERO, "original"),
        ] {
            let mut args = vec![("mode", mode), ("control_signature", "x")];
            let mut scenario = two_movers(demand);
            if profile == "original" {
                args.push(("profile", "original"));
                scenario.push_str(", fixture_api = true, query_original = true");
            }
            let driven = drive(&scenario, &args);
            // nothing was latched or run, and the finite counts are in the row and the reason
            assert!(driven.query_calls.is_empty(), "{mode} {profile}: {:?}", driven.query_calls);
            let selected = row_of(&driven, "query_case_selected");
            assert_eq!(
                (
                    selected["target"].clone(),
                    selected["moving_candidates"].clone(),
                    selected["speed_input_candidates"].clone(),
                    selected["selected_demand"].clone()
                ),
                (Value::from("none"), Value::from(2), Value::from(0), Value::from("none")),
                "{mode} {profile}: {selected}"
            );
            let error = error_of(driven);
            assert!(
                error.contains("UNREACHED: 2 alive routed vehicles") && error.contains("none of those move steps carried a finite positive desired_velocity"),
                "{mode} {profile}: {error}"
            );
        }
    }
}

#[test]
fn the_zero_demand_b_delegates_one_copy_that_differs_only_in_desired_velocity_and_is_credited_by_a_different_physical_result() {
    let control = drive(&demand_movers(""), &[("mode", SPEED_CONTROL)]);
    assert!(control.result.is_ok(), "{:?}\n{}", control.result, control.rows);
    let signature = signature_of(&control);
    let driven = drive(&demand_movers(""), &[("mode", SPEED_ZERO), ("control_signature", signature)]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(row_of(&driven, "query_prepared")["target"], "eech-world:2");
    let row = query_row(&driven);
    assert_eq!(
        (
            row["speed_window_valid"].clone(),
            row["speed_window_bad"].clone(),
            row["speed_transforms"].clone(),
            row["speed_delegations"].clone(),
            row["speed_incoming_demand"].clone(),
            row["speed_substituted_demand"].clone(),
            row["window_target_moves"].clone()
        ),
        (
            Value::from(1),
            Value::from(0),
            Value::from(1),
            Value::from(1),
            Value::from("7.5"),
            Value::from("0"),
            Value::from(1)
        ),
        "{row}"
    );
    // the target was offered after the delegated step and the real result is not the control's: it followed the demand the host's solver received
    assert!(row["target_after_speed"].as_i64().unwrap() >= 1, "{row}");
    assert_eq!(row["query_hits"], 0);
    assert_ne!(row["query_signature"].as_str().unwrap(), signature);
    // the host received exactly ONE changed step for the target in the query frame, carrying demand 0 and differing from the engine's own step in that one field; the engine gave 7.5 and its table is intact
    assert_eq!(
        demand_steps(&driven, "eech-world:2", "query"),
        vec![vec!["eech-world:2", "0", "query", "desired_velocity", "7.5"]]
    );
    // every other step (the target's warmup steps, the other vehicle's steps in both phases) reached the host as the engine's own table
    assert!(demand_steps(&driven, "eech-world:2", "warmup")
        .iter()
        .all(|fields| fields[3] == "-" && fields[1] == "7.5"));
    for phase in ["warmup", "query"] {
        assert!(!demand_steps(&driven, "eech-world:3", phase).is_empty(), "{phase}");
        assert!(
            demand_steps(&driven, "eech-world:3", phase)
                .iter()
                .all(|fields| fields[3] == "-" && fields[1] == "6"),
            "{phase}"
        );
    }
    assert!(!driven.caller_mutated);
    // the control keeps its own demand: the same target's query-frame step reached the host unchanged there
    assert_eq!(
        demand_steps(&control, "eech-world:2", "query"),
        vec![vec!["eech-world:2", "7.5", "query", "-", "7.5"]]
    );
}

#[test]
fn the_zero_demand_b_may_skip_the_narrow_phase_but_never_the_offer_and_an_unchanged_result_is_rejected() {
    // a valid broad-phase skip of the changed target is allowed once it was offered and answered
    let skipping = demand_movers(", candidate_sequence = { 'skip' }");
    let control = drive(&demand_movers(""), &[("mode", SPEED_CONTROL)]);
    let signature = signature_of(&control);
    let driven = drive(&skipping, &[("mode", SPEED_ZERO), ("control_signature", signature)]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = query_row(&driven);
    assert_eq!(
        (row["target_need_updated"].clone(), row["target_test"].clone()),
        (Value::from(0), Value::from(0)),
        "{row}"
    );
    assert!(row["target_after_speed"].as_i64().unwrap() >= 1);
    // while A keeps its strict moving predicates where B may skip
    let error = error_of(drive(&skipping, &[("mode", SPEED_CONTROL)]));
    assert!(
        error.contains("UNREACHED: moving_query_speed_control") && error.contains("never asked need_updated"),
        "{error}"
    );
    // the changed demand that leaves every output as A's is UNCHANGED, never credited
    let unchanged = two_movers(", demand = { [2] = 7.5, [3] = 6 }");
    let a = drive(&unchanged, &[("mode", SPEED_CONTROL)]);
    let error = error_of(drive(&unchanged, &[("mode", SPEED_ZERO), ("control_signature", signature_of(&a))]));
    assert!(error.contains("UNCHANGED") && error.contains("changed no output"), "{error}");
    // a changed result with no control signature is not compared, so it is not closed
    let error = error_of(drive(&demand_movers(""), &[("mode", SPEED_ZERO)]));
    assert!(error.contains("needs control_signature="), "{error}");
    // no question about the changed target after its step is UNREACHED, whatever the earlier counts were
    let error = error_of(drive(
        &demand_movers(", candidate_id = 'eech-world:3'"),
        &[("mode", SPEED_ZERO), ("control_signature", signature)],
    ));
    assert!(error.contains("UNREACHED") && error.contains("followed the changed-demand step"), "{error}");
}

#[test]
fn a_query_frame_step_without_a_valid_demand_or_no_step_at_all_is_unreached_and_never_credited() {
    let control = drive(&demand_movers(""), &[("mode", SPEED_CONTROL)]);
    let signature = signature_of(&control);
    // the target was eligible at the final warmup frame, but its query-frame step carries no usable demand: nothing is replaced and the reason carries the finite window counts
    for bad in ["'missing'", "0", "-3", "0/0", "1/0"] {
        let driven = drive(
            &demand_movers(&format!(", query_demand = {{ [2] = {bad} }}")),
            &[("mode", SPEED_ZERO), ("control_signature", signature)],
        );
        let row = query_row(&driven);
        assert_eq!(
            (
                row["speed_window_valid"].clone(),
                row["speed_window_bad"].clone(),
                row["speed_transforms"].clone(),
                row["speed_delegations"].clone()
            ),
            (Value::from(0), Value::from(1), Value::from(0), Value::from(0)),
            "{bad}: {row}"
        );
        assert_eq!(row["speed_incoming_demand"], "none");
        // the one query-frame step reached the host as the engine's own table
        let steps = demand_steps(&driven, "eech-world:2", "query");
        assert!(steps.len() == 1 && steps[0][3] == "-", "{bad}: {steps:?}");
        let error = error_of(driven);
        assert!(
            error.contains("UNREACHED: moving_query_speed_zero changed no step")
                && error.contains("the target was given 1 steps, 1 of mode move, 0 with a finite positive desired_velocity, 1 without"),
            "{bad}: {error}"
        );
    }
    // no callback of the target in the query frame at all
    let driven = drive(
        &demand_movers(", no_query_frame_steps = true"),
        &[("mode", SPEED_ZERO), ("control_signature", signature)],
    );
    let row = query_row(&driven);
    assert_eq!(
        (row["window_target_steps"].clone(), row["speed_transforms"].clone()),
        (Value::from(0), Value::from(0)),
        "{row}"
    );
    assert!(demand_steps(&driven, "eech-world:2", "query").is_empty());
    let error = error_of(driven);
    assert!(
        error.contains("UNREACHED: moving_query_speed_zero changed no step") && error.contains("the target was given 0 steps"),
        "{error}"
    );
    // while the unchanged A does not need a window step (it replaces nothing)
    let driven = drive(&demand_movers(", no_query_frame_steps = true"), &[("mode", SPEED_CONTROL)]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
}

#[test]
fn the_original_profile_receives_the_same_changed_demand_and_asks_the_host_nothing() {
    let original = |mode: &'static str, signature: &'static str| {
        let mut args = vec![("profile", "original"), ("mode", mode)];
        if !signature.is_empty() {
            args.push(("control_signature", signature));
        }
        args
    };
    let original_scenario = |extra: &str| demand_movers(&format!(", fixture_api = true, query_original = true{extra}"));
    let a = drive(&original_scenario(""), &original(SPEED_CONTROL, ""));
    assert!(a.result.is_ok(), "{:?}\n{}", a.result, a.rows);
    let signature = signature_of(&a);
    let b = drive(&original_scenario(""), &original(SPEED_ZERO, signature));
    assert!(b.result.is_ok(), "{:?}\n{}", b.result, b.rows);
    // the same actor, the same one-frame input change as the hosted B, and no hosted question of any kind
    assert_eq!(row_of(&b, "query_prepared")["target"], "eech-world:2");
    let row = query_row(&b);
    assert_eq!(
        (
            row["entity_after"].clone(),
            row["candidate_after"].clone(),
            row["scan_after"].clone(),
            row["pose_after"].clone(),
            row["speed_transforms"].clone(),
            row["speed_delegations"].clone()
        ),
        (Value::from(0), Value::from(0), Value::from(0), Value::from(0), Value::from(1), Value::from(1)),
        "{row}"
    );
    assert_eq!(
        demand_steps(&b, "eech-world:2", "query"),
        vec![vec!["eech-world:2", "0", "query", "desired_velocity", "7.5"]]
    );
    assert_ne!(row["query_signature"].as_str().unwrap(), signature);
    // a bound control the changed demand does not differ from is UNCHANGED, and a step it cannot replace is UNREACHED
    let unchanged = two_movers(", demand = { [2] = 7.5, [3] = 6 }, fixture_api = true, query_original = true");
    let same = drive(
        &unchanged,
        &original(SPEED_ZERO, signature_of(&drive(&unchanged, &original(SPEED_CONTROL, "")))),
    );
    assert!(error_of(same).contains("UNCHANGED"));
    let error = error_of(drive(&original_scenario(", no_query_frame_steps = true"), &original(SPEED_ZERO, signature)));
    assert!(error.contains("UNREACHED: moving_query_speed_zero changed no step"), "{error}");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the FIXED_STATE reports (phase 13, Task49: the host's record of a placement holds a fixed entity's collision state) and the placement key every staged question of a non-mobile candidate names. The real
// provider answers the report and keeps the state in its real roster; the scripted collaborators state a candidate question's refusal for a withdrawn state in the provider's own words (the provider's
// own refusal is pinned by tests/model_target_point.rs, and the native run asks the real provider), reading what the REAL roster holds. `withdrawn_fixed_state` is the host-side mode

/// a fixed candidate of the sector placement: the real provider holds the report, and the candidate's three staged questions name the placement
const FIXED_STATE: &str = ", real_scan = true, fixed_state = true, fixed_candidate = true, candidate_sequence = { 'need_fixed', 'need_placement', 'test' }";

#[test]
fn a_healthy_control_counts_the_fixed_state_reports_apart_and_every_staged_question_of_a_non_mobile_candidate_names_the_placement() {
    let (driven, row) = scan_row(FIXED_STATE, "scan_real");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    for (field, expected) in [
        ("request_faults", 0),
        ("reply_faults", 0),
        // the report is neither a roster event nor a city event nor a scan question: it is counted apart, and acknowledged once
        ("fixed_before", 0),
        ("fixed_after", 1),
        ("fixed_total", 1),
        ("fixed_stated", 1),
        ("roster_after", 2),
        ("city_after", 0),
        ("scan_after", 3),
        // the three staged questions each name the placement; the two that needed the state (the fixed facts read, then the placement) are the eligible lookups
        ("candidate_keyed", 3),
        ("candidate_unkeyed", 0),
        ("fixed_joined", 2),
        ("fixed_withdrawn", 0),
        ("candidate_altered", 0),
    ] {
        assert_eq!(row[field], expected, "{field}: {row}");
    }
    assert_eq!(
        driven.fixed_log,
        [("0x41500000".to_string(), "placed".to_string(), "1".to_string(), "1".to_string())],
        "the host acknowledged the state of the placement it had just issued"
    );
    assert_eq!(driven.scan_log.len(), 3, "the report is not a scan question");
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);
}

#[test]
fn withdrawing_the_fixed_state_of_a_served_placement_makes_the_next_question_of_the_same_candidate_refused_by_name() {
    let driven = drive(&healthy(FIXED_STATE), &[("mode", "withdrawn_fixed_state")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = authority(&driven);
    assert_eq!(row["completed"], false);
    assert_eq!(row["failure_marker"], "entity_candidate.refused");
    // the eligible lookup was served over the held state BEFORE the negative (once), the host's own withdrawal was acknowledged `removed`, and the next question was the same candidate's placement question
    assert_eq!(
        (
            row["fixed_joined"].clone(),
            row["fixed_withdrawn"].clone(),
            row["fixed_withdrawn_next"].clone(),
            row["fixed_withdrawn_cause"].clone()
        ),
        (Value::from(1), Value::from(1), Value::from("placement"), Value::from("fixed_state_withdrawn")),
        "{row}"
    );
    // no answer and no request was altered: the only fault is the provider's named refusal, and the questions asked are the three of the dialogue, the last of them refused
    assert_eq!(
        (
            row["candidate_altered"].clone(),
            row["altered"].clone(),
            row["scan_altered"].clone(),
            row["request_faults"].clone(),
            row["reply_faults"].clone()
        ),
        (Value::from(0), Value::from(0), Value::from(0), Value::from(0), Value::from(1))
    );
    assert_eq!(row["first_fault"], "candidate reply: result refused");
    assert_eq!(
        (row["candidate_after"].clone(), row["entity_after"].clone()),
        (Value::from(3), Value::from(0)),
        "nothing was asked after the refusal"
    );
    // the real provider acknowledged the campaign's report and then the host's withdrawal of ONLY that record's state, in that order, for the one placement
    assert_eq!(
        driven.fixed_log,
        [
            ("0x41500000".to_string(), "placed".to_string(), "1".to_string(), "1".to_string()),
            ("0x41500000".to_string(), "removed".to_string(), String::new(), "1".to_string()),
        ]
    );
    assert_eq!(
        driven.roster_log.len(),
        2,
        "the placement itself stayed standing: only the boot and the insertion are roster events"
    );
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);

    // the prerequisite is not a pass when it was never reached, and honestly missing when nothing failed: a candidate no question of which was answered `need_placement` and a placement the host was never told a
    // state for are UNREACHED (the run itself was clean)
    for extra in [
        ", real_scan = true, fixed_state = true, fixed_candidate = true, candidate_sequence = { 'skip' }",
        ", real_scan = true, fixed_candidate = true, candidate_sequence = { 'need_fixed', 'need_placement', 'test' }",
    ] {
        let error = error_of(drive(&healthy(extra), &[("mode", "withdrawn_fixed_state")]));
        assert!(error.contains("UNREACHED") && error.contains("holds no such candidate"), "{extra}: {error}");
    }
    // a failure of the run itself before any eligible lookup was answered is that failure, never an absent prerequisite: an engine stop before any question, a request fault and a reply fault
    let error = error_of(drive(
        &healthy(&format!("{FIXED_STATE}, engine_stop = {{ after = 1, marker = 'entity_candidate.callback' }}")),
        &[("mode", "withdrawn_fixed_state")],
    ));
    assert!(
        !error.contains("UNREACHED")
            && error.contains("stopped at [model-provider:entity_candidate.callback], not at [model-provider:entity_candidate.refused]"),
        "{error}"
    );
    let error = error_of(drive(
        &healthy(&format!("{FIXED_STATE}, no_candidate_key = true")),
        &[("mode", "withdrawn_fixed_state")],
    ));
    assert!(
        !error.contains("UNREACHED") && error.contains("request_faults=") && error.contains("is not a placement key"),
        "{error}"
    );
    let error = error_of(drive(
        &healthy(&format!("{FIXED_STATE}, bad_real_candidate = 'state'")),
        &[("mode", "withdrawn_fixed_state")],
    ));
    assert!(
        !error.contains("UNREACHED") && error.contains("reply_faults=1") && error.contains("candidate reply:"),
        "{error}"
    );
    let error = error_of(drive(
        &healthy(", fixture_api = true"),
        &[("profile", "original"), ("mode", "withdrawn_fixed_state")],
    ));
    assert!(error.contains("only mode=control"), "{error}");
}

#[test]
fn changing_the_physical_rule_of_a_served_placement_makes_the_next_unmodified_question_refused_by_the_mirror_difference() {
    let driven = drive(&healthy(FIXED_STATE), &[("mode", "changed_fixed_rule")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = authority(&driven);
    assert_eq!(row["completed"], false);
    assert_eq!(row["failure_marker"], "entity_candidate.refused");
    // the eligible lookup (answered need_placement over the record's collision-fixed rule) was counted ONCE before the change, the restatement was acknowledged, and the next question was the same placement's
    assert_eq!(
        (
            row["fixed_rule_eligible"].clone(),
            row["fixed_rule_changed"].clone(),
            row["fixed_rule_changed_next"].clone(),
            row["fixed_rule_changed_cause"].clone()
        ),
        (
            Value::from(1),
            Value::from(1),
            Value::from("placement"),
            Value::from("fixed_rule_mirror_differs")
        ),
        "{row}"
    );
    // no answer and no request was altered: the only fault is the provider's named refusal, and the questions are the dialogue's three, the last refused
    assert_eq!(
        (
            row["candidate_altered"].clone(),
            row["altered"].clone(),
            row["scan_altered"].clone(),
            row["request_faults"].clone(),
            row["reply_faults"].clone()
        ),
        (Value::from(0), Value::from(0), Value::from(0), Value::from(0), Value::from(1))
    );
    assert_eq!(row["first_fault"], "candidate reply: result refused");
    assert_eq!(
        (row["candidate_after"].clone(), row["entity_after"].clone()),
        (Value::from(3), Value::from(0)),
        "nothing was asked after the refusal"
    );
    // the real provider acknowledged the campaign's report and then the host's restatement of ONLY that record's state, in that order, both `placed` for the one placement; the placement stayed standing
    assert_eq!(
        driven.fixed_log,
        [
            ("0x41500000".to_string(), "placed".to_string(), "1".to_string(), "1".to_string()),
            ("0x41500000".to_string(), "placed".to_string(), "1".to_string(), "1".to_string()),
        ]
    );
    assert_eq!(driven.roster_log.len(), 2, "only the boot and the insertion are roster events");
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);
    // the prerequisite is not a pass when it was never reached: a candidate no question of which was answered need_placement is UNREACHED (the run itself was clean)
    let error = error_of(drive(
        &healthy(", real_scan = true, fixed_state = true, fixed_candidate = true, candidate_sequence = { 'skip' }"),
        &[("mode", "changed_fixed_rule")],
    ));
    assert!(error.contains("UNREACHED") && error.contains("holds no such candidate"), "{error}");
    // a run that failed first is that failure, never an absent prerequisite
    let error = error_of(drive(
        &healthy(&format!("{FIXED_STATE}, engine_stop = {{ after = 1, marker = 'entity_candidate.callback' }}")),
        &[("mode", "changed_fixed_rule")],
    ));
    assert!(
        !error.contains("UNREACHED")
            && error.contains("stopped at [model-provider:entity_candidate.callback], not at [model-provider:entity_candidate.refused]"),
        "{error}"
    );
}

#[test]
fn a_restatement_acknowledged_with_a_cursor_beside_placed_and_the_same_key_is_never_credited_as_a_change() {
    // positive first: the same scenario with a correct acknowledgement is credited (the test above)
    let positive = drive(&healthy(FIXED_STATE), &[("mode", "changed_fixed_rule")]);
    assert!(positive.result.is_ok(), "{:?}\n{}", positive.result, positive.rows);
    assert_eq!(authority(&positive)["fixed_rule_changed"], 1);
    // only the restatement's acknowledgement is damaged (the initial report is acknowledged exactly as before); the fixed-report reply contract forbids a cursor with `placed`
    let driven = drive(
        &healthy(&format!("{FIXED_STATE}, bad_real_fixed = 'restatement_cursor'")),
        &[("mode", "changed_fixed_rule")],
    );
    let row = authority(&driven);
    assert_eq!(
        driven.fixed_log,
        [
            ("0x41500000".to_string(), "placed".to_string(), "1".to_string(), "1".to_string()),
            ("0x41500000".to_string(), "placed".to_string(), "1".to_string(), "1".to_string()),
        ],
        "the report and the restatement were both acknowledged by the provider: only the restatement's reply carries the cursor"
    );
    assert_eq!(
        (
            row["fixed_rule_eligible"].clone(),
            row["fixed_rule_changed"].clone(),
            row["request_faults"].clone(),
            row["first_fault"].clone()
        ),
        (
            Value::from(1),
            Value::from(-1),
            Value::from(0),
            Value::from("candidate reply: the host's restatement of the fixed rule of the placement was not acknowledged as placed")
        ),
        "{row}"
    );
    let error = error_of(driven);
    assert!(
        error.contains("mode changed_fixed_rule") && error.contains("restatement of the fixed rule") && error.contains("not acknowledged as placed"),
        "{error}"
    );
    assert!(!error.contains("UNREACHED") && !error.contains("completed"), "{error}");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the changed physical rule at the FINAL placement question (Task49 witness correction). The native attempt on the ordinary-need_placement selector was UNREACHED (no ordinary fixed placement was ever answered
// need_placement), while a building listed under a city has consumed its held sub-type-derived collision flag when its placement question yields the final pose (`fixed_join`: for `of_city`, the flag is compared once
// the placement is given). The mode therefore delegates the campaign's UNMODIFIED final-placement question to the same provider first, then restates ONLY that record's sub type as UNKNOWN, and asks the provider the SAME
// payload once more (an extra Host-only evaluation across two controlled Host states, never a claim that the campaign asked twice) and hands the provider's own refusal back.

/// a city whose own state the campaign reports and ONE building listed under it whose fixed state it reports too: the building's own dialogue reads its fixed facts, its placement, its normal and ends in the final pose
const LISTED: &str = ", real_scan = true, city_state = true, building_state = true, fixed_candidate = true, city = 1, candidate_sequence = { 'need_fixed', 'buildings', 'need_fixed', 'need_placement', 'need_normal', 'test' }";

const MIRROR: &str = "[candidate.mirror] the campaign's collision-fixed flag differs from the host's fixed state of the placement";

const LISTED_DIALOGUE: [&str; 6] = ["need_fixed", "buildings", "need_fixed", "need_placement", "need_normal", "test"];

#[test]
fn a_listed_buildings_final_placement_question_is_re_evaluated_over_its_restated_sub_type_and_the_providers_mirror_refusal_is_handed_back() {
    // positive first: the unchanged control ends in the final pose of the building, nothing is restated, the four rule fields hold their declared zeros and the dialogue is the one the mode must reproduce
    let (control, row) = scan_row(LISTED, "scan_real");
    assert!(control.result.is_ok(), "{:?}\n{}", control.result, control.rows);
    assert_eq!(row["completed"], true, "{row}");
    assert_eq!(
        (
            row["fixed_rule_eligible"].clone(),
            row["fixed_rule_changed"].clone(),
            row["fixed_rule_changed_next"].clone(),
            row["fixed_rule_changed_cause"].clone(),
            row["request_faults"].clone(),
            row["reply_faults"].clone(),
            row["candidate_after"].clone()
        ),
        (
            Value::from(0),
            Value::from(0),
            Value::from(""),
            Value::from(""),
            Value::from(0),
            Value::from(0),
            Value::from(6)
        ),
        "{row}"
    );
    let dialogue: Vec<&str> = control.candidate_calls.iter().map(|call| call.2.as_str()).collect();
    assert_eq!(dialogue, LISTED_DIALOGUE, "the building's dialogue reached its final pose");

    let driven = drive(&healthy(LISTED), &[("mode", "changed_fixed_rule")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = authority(&driven);
    assert_eq!(row["completed"], false);
    assert_eq!(row["failure_marker"], "entity_candidate.refused");
    // the final pose was validated BEFORE the change (eligible), the restatement of ONLY that record's sub type was acknowledged, the SAME payload was re-evaluated, and the provider's own named mirror refusal is the cause
    assert_eq!(
        (
            row["fixed_rule_eligible"].clone(),
            row["fixed_rule_changed"].clone(),
            row["fixed_rule_changed_next"].clone(),
            row["fixed_rule_changed_cause"].clone()
        ),
        (
            Value::from(1),
            Value::from(1),
            Value::from("placement"),
            Value::from("fixed_rule_mirror_differs")
        ),
        "{row}"
    );
    // no campaign answer or request was altered: the only fault is the provider's named refusal, handed back for the ONE question the campaign asked at that stage. The campaign asked six questions, never a seventh:
    // the re-evaluation is the host's own and is not counted as one
    assert_eq!(
        (
            row["candidate_altered"].clone(),
            row["altered"].clone(),
            row["scan_altered"].clone(),
            row["request_faults"].clone(),
            row["reply_faults"].clone(),
            row["candidate_after"].clone(),
            row["entity_after"].clone()
        ),
        (
            Value::from(0),
            Value::from(0),
            Value::from(0),
            Value::from(0),
            Value::from(1),
            Value::from(6),
            Value::from(0)
        ),
        "{row}"
    );
    assert_eq!(row["first_fault"], "candidate reply: result refused");
    // the provider received the campaign's six questions exactly as the control's, a SEVENTH call that is the sixth's payload byte for byte (same scene, same text, same key) and its refusal by name
    assert_eq!(driven.candidate_calls.len(), 7, "{:?}", driven.candidate_calls);
    assert_eq!(
        driven.candidate_calls[..6],
        control.candidate_calls[..],
        "the six questions and their real answers are the control's"
    );
    let (final_question, again) = (&driven.candidate_calls[5], &driven.candidate_calls[6]);
    assert_eq!(
        final_question.2, "test",
        "the final pose was answered over the held state before anything was restated"
    );
    assert_eq!(
        (again.0, &again.1),
        (final_question.0, &final_question.1),
        "the same payload to the same provider"
    );
    assert_eq!((again.2.as_str(), again.3.as_str()), ("refused", MIRROR));
    // the provider acknowledged the building's own report and then the host's restatement of ONLY that record, both `placed` under the building's key
    let child = final_question.1.rsplit(' ').next().expect("the building's key").to_string();
    let reports: Vec<_> = driven.fixed_log.iter().filter(|entry| entry.0 == "0x41500000").collect();
    assert_eq!(
        reports,
        [
            &("0x41500000".to_string(), "placed".to_string(), child.clone(), child.clone()),
            &("0x41500000".to_string(), "placed".to_string(), child.clone(), child.clone()),
        ]
    );
    // the extra evaluation took no destructive, terrain, motion or engine action: nothing reached the engine, the movement registry or the controlled query surface
    assert!(
        driven.entity_log.is_empty()
            && driven.move_log.is_empty()
            && driven.query_calls.is_empty()
            && driven.demand_log.is_empty()
            && driven.pose_log.is_empty(),
        "{:?} {:?} {:?}",
        driven.entity_log,
        driven.move_log,
        driven.query_calls
    );
    assert!(
        driven.roster_log.iter().all(|event| event.0 != "0x40a00000"),
        "the placement stayed standing: no roster removal"
    );
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);
}

#[test]
fn a_restatement_not_acknowledged_under_the_fixed_report_contract_at_a_listed_buildings_final_question_is_never_credited_and_the_payload_is_never_re_evaluated()
{
    // positive first: the same scenario with a correct acknowledgement is credited (the test above). Only the restatement's acknowledgement is damaged: a cursor beside `placed`, another placement's key, another
    // state, and a provider that fails to answer at all
    for fault in ["restatement_cursor", "restatement_echo", "restatement_state", "restatement_error"] {
        let driven = drive(&healthy(&format!("{LISTED}, bad_real_fixed = '{fault}'")), &[("mode", "changed_fixed_rule")]);
        let row = authority(&driven);
        assert_eq!(
            (
                row["fixed_rule_eligible"].clone(),
                row["fixed_rule_changed"].clone(),
                row["fixed_rule_changed_next"].clone(),
                row["fixed_rule_changed_cause"].clone(),
                row["request_faults"].clone()
            ),
            (Value::from(1), Value::from(-1), Value::from(""), Value::from(""), Value::from(0)),
            "{fault}: {row}"
        );
        assert_eq!(
            row["first_fault"], "candidate reply: the host's restatement of the fixed rule of the placement was not acknowledged as placed",
            "{fault}"
        );
        // the final pose is the campaign's last answer: nothing was asked of the provider after the failed restatement, and no refusal was handed back
        let dialogue: Vec<&str> = driven.candidate_calls.iter().take(6).map(|call| call.2.as_str()).collect();
        assert_eq!(dialogue, LISTED_DIALOGUE, "{fault}");
        assert!(
            driven.candidate_calls.len() >= 6 && driven.candidate_calls.iter().skip(6).all(|call| call.1 != driven.candidate_calls[5].1),
            "{fault}: the payload was re-asked"
        );
        let error = error_of(driven);
        assert!(
            error.contains("mode changed_fixed_rule") && error.contains("restatement of the fixed rule") && error.contains("not acknowledged as placed"),
            "{fault}: {error}"
        );
        assert!(!error.contains("UNREACHED") && !error.contains("completed"), "{fault}: {error}");
    }
}

#[test]
fn a_listed_building_that_never_reaches_the_final_pose_or_a_run_that_failed_first_is_never_a_pass_of_the_changed_rule() {
    // the dialogue ends in `skip` at the placement stage: no final pose was ever answered, so there was no consumed flag to change
    let skipped = ", real_scan = true, city_state = true, building_state = true, fixed_candidate = true, city = 1, candidate_sequence = { 'need_fixed', 'buildings', 'need_fixed', 'need_placement', 'skip' }";
    let error = error_of(drive(&healthy(skipped), &[("mode", "changed_fixed_rule")]));
    assert!(error.contains("UNREACHED") && error.contains("holds no such candidate"), "{error}");
    // a building the campaign never stated a fixed state for (the provider holds none): its final question is not an eligible lookup
    let stateless = ", real_scan = true, city_state = true, fixed_candidate = true, city = 1, candidate_sequence = { 'need_fixed', 'buildings', 'need_fixed', 'need_placement', 'need_normal', 'test' }";
    let error = error_of(drive(&healthy(stateless), &[("mode", "changed_fixed_rule")]));
    assert!(error.contains("UNREACHED") && error.contains("holds no such candidate"), "{error}");
    // a run that failed first is that failure, never an absent prerequisite: an engine stop before any question, and a reply fault of the real provider
    let error = error_of(drive(
        &healthy(&format!("{LISTED}, engine_stop = {{ after = 1, marker = 'entity_candidate.callback' }}")),
        &[("mode", "changed_fixed_rule")],
    ));
    assert!(
        !error.contains("UNREACHED")
            && error.contains("stopped at [model-provider:entity_candidate.callback], not at [model-provider:entity_candidate.refused]"),
        "{error}"
    );
    let error = error_of(drive(
        &healthy(&format!("{LISTED}, bad_real_candidate = 'state'")),
        &[("mode", "changed_fixed_rule")],
    ));
    assert!(
        !error.contains("UNREACHED") && error.contains("reply_faults=1") && error.contains("candidate reply:"),
        "{error}"
    );
}

/// ONE fault in the intercepted final positive: the campaign's consumer contract (a nonzero scene, exactly twelve finite words, the scene's own name) is checked BEFORE any credit, restatement or extra evaluation, so the
/// campaign meets its own refusal of the original and nothing is hidden behind the counterfactual refusal
fn a_single_faulty_positive_is_neither_credited_nor_restated_nor_re_evaluated(extra: &str, what: &str) {
    let driven = drive(&healthy(&format!("{LISTED}{extra}")), &[("mode", "changed_fixed_rule")]);
    let row = authority(&driven);
    assert_eq!(
        (
            row["fixed_rule_eligible"].clone(),
            row["fixed_rule_changed"].clone(),
            row["fixed_rule_changed_next"].clone(),
            row["fixed_rule_changed_cause"].clone(),
            row["request_faults"].clone(),
            row["reply_faults"].clone()
        ),
        (Value::from(0), Value::from(0), Value::from(""), Value::from(""), Value::from(0), Value::from(1)),
        "{extra}: {row}"
    );
    assert!(
        row["first_fault"]
            .as_str()
            .is_some_and(|fault| fault.starts_with("candidate reply: ") && fault.contains(what)),
        "{extra}: {row}"
    );
    // the building's own report is the only fixed report (no restatement) and the provider was asked the six questions of the campaign and nothing more (no re-evaluation, no refusal)
    assert_eq!(driven.fixed_log.iter().filter(|entry| entry.0 == "0x41500000").count(), 1, "{extra}");
    assert_eq!(driven.candidate_calls.len(), 6, "{extra}: {:?}", driven.candidate_calls);
    assert!(driven.candidate_calls.iter().all(|call| call.2 != "refused"), "{extra}");
    let error = error_of(driven);
    assert!(
        !error.contains("UNREACHED") && error.contains("no rule was changed because the run failed first") && error.contains(what),
        "{extra}: {error}"
    );
}

#[test]
fn valid_final_positives_including_a_subnormal_word_and_a_negative_zero_are_credited_and_refused_by_name() {
    // positive first: the coherent positive, and the valid corner words a float conversion would damage, are intercepted, credited once and answered by the provider's refusal
    for valid in ["", ", final_fault = 'subnormal'"] {
        let driven = drive(&healthy(&format!("{LISTED}{valid}")), &[("mode", "changed_fixed_rule")]);
        assert!(driven.result.is_ok(), "{valid}: {:?}\n{}", driven.result, driven.rows);
        let row = authority(&driven);
        assert_eq!(
            (
                row["fixed_rule_eligible"].clone(),
                row["fixed_rule_changed"].clone(),
                row["reply_faults"].clone()
            ),
            (Value::from(1), Value::from(1), Value::from(1)),
            "{valid}: {row}"
        );
        assert_eq!(driven.candidate_calls.len(), 7, "{valid}");
    }
}

#[test]
fn a_final_positive_with_an_infinite_pose_word_is_never_credited_and_causes_no_restatement_and_no_re_evaluation() {
    a_single_faulty_positive_is_neither_credited_nor_restated_nor_re_evaluated(", final_fault = 'inf'", "is not a finite number");
    a_single_faulty_positive_is_neither_credited_nor_restated_nor_re_evaluated(", final_fault = 'ninf'", "is not a finite number");
}

#[test]
fn a_final_positive_with_a_signaling_nan_pose_word_is_never_credited_and_causes_no_restatement_and_no_re_evaluation() {
    a_single_faulty_positive_is_neither_credited_nor_restated_nor_re_evaluated(", final_fault = 'snan'", "is not a finite number");
}

#[test]
fn a_final_positive_for_scene_zero_is_never_credited_and_causes_no_restatement_and_no_re_evaluation() {
    a_single_faulty_positive_is_neither_credited_nor_restated_nor_re_evaluated(", final_scene = 'zero'", "nonzero scene");
}

#[test]
fn a_final_positive_that_names_another_scene_is_never_credited_and_causes_no_restatement_and_no_re_evaluation() {
    a_single_faulty_positive_is_neither_credited_nor_restated_nor_re_evaluated(", final_fault = 'name'", "names the scene");
}

#[test]
fn a_final_positive_for_a_scene_no_accepted_reply_ever_named_is_not_validated_and_is_neither_credited_nor_restated_nor_re_evaluated() {
    // the name the campaign expects cannot be known for a scene no reply it accepted ever named: no credit, no restatement, no re-evaluation; it is neither a pass nor a provider fault
    let driven = drive(&healthy(&format!("{LISTED}, final_scene = 'unbound'")), &[("mode", "changed_fixed_rule")]);
    let row = authority(&driven);
    assert_eq!(
        (
            row["fixed_rule_eligible"].clone(),
            row["fixed_rule_changed"].clone(),
            row["request_faults"].clone(),
            row["reply_faults"].clone()
        ),
        (Value::from(0), Value::from(0), Value::from(0), Value::from(0)),
        "{row}"
    );
    assert_eq!(driven.fixed_log.iter().filter(|entry| entry.0 == "0x41500000").count(), 1);
    assert_eq!(driven.candidate_calls.len(), 6);
    let error = error_of(driven);
    assert!(error.contains("UNREACHED") && error.contains("not validated"), "{error}");
}

#[test]
fn a_wrong_served_key_the_real_provider_could_answer_is_neither_credited_as_an_eligible_lookup_nor_withdrawn() {
    // the real provider holds a state for a SECOND live placement and the candidate names it, although the host served the first: the real provider could answer that question over a real held state, but the
    // key is not the placement the host actually served, so the driver counts the fault, credits no eligible lookup and withdraws nothing
    let extra = format!("{FIXED_STATE}, second_held = true, candidate_second = true");
    let driven = drive(&healthy(&extra), &[("mode", "withdrawn_fixed_state")]);
    let row = authority(&driven);
    assert_eq!(
        driven.fixed_log,
        [
            ("0x41500000".to_string(), "placed".to_string(), "1".to_string(), "1".to_string()),
            ("0x41500000".to_string(), "placed".to_string(), "2".to_string(), "2".to_string()),
        ],
        "both placements were stated and nothing was removed"
    );
    assert_eq!(
        (row["fixed_joined"].clone(), row["fixed_withdrawn"].clone(), row["fixed_withdrawn_next"].clone()),
        (Value::from(0), Value::from(0), Value::from("")),
        "{row}"
    );
    assert_eq!((row["candidate_altered"].clone(), row["altered"].clone()), (Value::from(0), Value::from(0)));
    assert!(row["request_faults"].as_i64().unwrap() >= 1, "{row}");
    let error = error_of(driven);
    assert!(
        !error.contains("UNREACHED")
            && error.contains("candidate request: the candidate names the placement 2, which is not the entry the host last served (1)"),
        "{error}"
    );
}

#[test]
fn each_single_fault_of_the_fixed_state_grammar_is_counted_at_its_question_and_is_never_a_pass() {
    // a non-mobile candidate that names a placement the host did NOT last serve (a live key of another entry) is a request fault too: the key must be the placement the host actually served
    let (driven, row) = scan_row(&format!("{FIXED_STATE}, candidate_key = '7'"), "scan_real");
    assert!(driven.result.is_err(), "{:?}\n{}", driven.result, driven.rows);
    assert!(
        row["first_fault"]
            .as_str()
            .unwrap()
            .contains("candidate request: the candidate names the placement 7, which is not the entry the host last served (1)"),
        "{row}"
    );
    // a non-mobile candidate that names no placement is a request fault (the provider refuses it by name, and the control that saw it fails)
    let (driven, row) = scan_row(&format!("{FIXED_STATE}, no_candidate_key = true"), "scan_real");
    assert!(driven.result.is_err(), "{:?}\n{}", driven.result, driven.rows);
    assert!(
        row["request_faults"].as_i64().unwrap() >= 1 && row["candidate_unkeyed"].as_i64().unwrap() >= 1,
        "{row}"
    );
    assert!(
        row["first_fault"]
            .as_str()
            .unwrap()
            .starts_with("candidate request: a non-mobile candidate names \"\""),
        "{row}"
    );
    // a report that states a fact it does not state, carries the host's own withdrawal word, names no key or a key that is none, is a request fault of its own question
    for (fault, expected) in [
        (
            "stray_fact",
            "states nothing but the position, the type, the heading, the shape, the sub type and its phase",
        ),
        (
            "withdraw_word",
            "states nothing but the position, the type, the heading, the shape, the sub type and its phase",
        ),
        // the campaign no longer states the flags it derived: a report that still does is a request fault
        (
            "stale_flag",
            "states nothing but the position, the type, the heading, the shape, the sub type and its phase",
        ),
        ("no_key", "names \"\", which is not a roster key"),
        ("bad_key", "names \"01\", which is not a roster key"),
    ] {
        let (driven, row) = scan_row(&format!("{FIXED_STATE}, bad_fixed_request = '{fault}'"), "scan_real");
        assert!(driven.result.is_err(), "{fault}: {:?}", driven.result);
        assert!(row["request_faults"].as_i64().unwrap() >= 1, "{fault}: {row}");
        let first = row["first_fault"].as_str().unwrap();
        assert!(first.starts_with("scan request:") && first.contains(expected), "{fault}: {first}");
    }
    // a reply that acknowledges another placement's key, or answers another state, is a reply fault
    for (fault, expected) in [("echo", "acknowledged \"99\" for the placement 1"), ("state", "answered removed, not placed")] {
        let (driven, row) = scan_row(&format!("{FIXED_STATE}, bad_real_fixed = '{fault}'"), "scan_real");
        assert!(driven.result.is_err(), "{fault}: {:?}", driven.result);
        assert!(row["reply_faults"].as_i64().unwrap() >= 1, "{fault}: {row}");
        let first = row["first_fault"].as_str().unwrap();
        assert!(first.starts_with("scan reply:") && first.contains(expected), "{fault}: {first}");
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// THE CITY'S OWN COLLISION STATE (phase 14, Task49): the host's record of a city's sector placement holds its position and bounding box, stated by the insertion and by each building's link (which expands the box),
// and a staged question of the city that reads them (its fixed facts read, fixed, a city, its first building served) is decided from the record. The real provider answers the reports and keeps the state in its real
// roster; the scripted collaborators state the candidate question's refusal for a withdrawn state in the provider's own words (the provider's own refusal is pinned by tests/model_target_point.rs), reading what the REAL
// roster holds. `withdrawn_city_state` is the host-side mode: see the driver's header.

/// a city of two buildings: the real provider holds the reports; the city's own questions are the fixed facts and its first building (the buildings' own dialogue follows)
const CITY_STATE: &str = ", real_scan = true, city_state = true, fixed_candidate = true, city = 2, candidate_sequence = { 'need_fixed', 'buildings', 'test' }";

#[test]
fn a_healthy_control_counts_the_city_state_reports_apart_and_the_city_stage_that_needs_the_state_is_an_eligible_lookup() {
    let (driven, row) = scan_row(CITY_STATE, "scan_real");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    for (field, expected) in [
        ("request_faults", 0),
        ("reply_faults", 0),
        // the insertion states the city once and each of its two buildings' links states the expanded box: three reports, counted apart from the family's
        ("city_state_before", 0),
        ("city_state_after", 3),
        ("city_state_total", 3),
        ("city_state_stated", 3),
        ("fixed_total", 0),
        ("fixed_stated", 0),
        // the one placement holds a state, and its staged questions all name it: the stage that reads the position and box is the one eligible lookup
        ("city_state_held_lifetime", 1),
        ("city_state_joined", 1),
        ("city_state_withdrawn", 0),
        ("candidate_altered", 0),
    ] {
        assert_eq!(row[field], expected, "{field}: {row}");
    }
    assert_eq!(
        driven.fixed_log.iter().map(|entry| entry.0.as_str()).collect::<Vec<_>>(),
        ["0x41600000", "0x41600000", "0x41600000"],
        "three reports, all of the city's phase"
    );
    assert!(
        driven.fixed_log.iter().all(|entry| entry.1 == "placed" && entry.2 == "1" && entry.3 == "1"),
        "{:?}",
        driven.fixed_log
    );
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);
}

#[test]
fn a_citys_sector_relink_states_the_new_placement_and_the_old_key_is_no_longer_a_held_one() {
    let (driven, row) = scan_row(&format!("{CITY_STATE}, city_relink = true"), "scan_real");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    // the insertion, the two buildings' links, and the new placement the relink inserted (its record continues the list's): four reports under two keys
    assert_eq!(
        (row["city_state_stated"].clone(), row["city_state_held_lifetime"].clone()),
        (Value::from(4), Value::from(2)),
        "{row}"
    );
    assert_eq!((row["request_faults"].clone(), row["reply_faults"].clone()), (Value::from(0), Value::from(0)));
    assert_eq!(row["city_state_joined"], 1, "the question names the placement the host serves now: {row}");
}

#[test]
fn withdrawing_the_city_state_of_a_served_placement_makes_the_next_question_that_reads_it_refused_by_name() {
    let extra = format!("{CITY_STATE}, city_state_again = true");
    let driven = drive(&healthy(&extra), &[("mode", "withdrawn_city_state")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = authority(&driven);
    assert_eq!(row["completed"], false);
    assert_eq!(row["failure_marker"], "entity_candidate.refused");
    // the eligible lookup was served over the held state BEFORE the negative (once), the host's own withdrawal was acknowledged `removed`, and the next question that read the city was the same placement's
    assert_eq!(
        (
            row["city_state_joined"].clone(),
            row["city_state_withdrawn"].clone(),
            row["city_state_withdrawn_next"].clone(),
            row["city_state_withdrawn_cause"].clone()
        ),
        (Value::from(1), Value::from(1), Value::from("city"), Value::from("city_state_withdrawn")),
        "{row}"
    );
    // no answer and no request was altered: the only fault is the provider's named refusal
    assert_eq!(
        (
            row["candidate_altered"].clone(),
            row["altered"].clone(),
            row["scan_altered"].clone(),
            row["request_faults"].clone(),
            row["reply_faults"].clone()
        ),
        (Value::from(0), Value::from(0), Value::from(0), Value::from(0), Value::from(1))
    );
    assert_eq!(row["first_fault"], "candidate reply: result refused");
    // the real provider acknowledged the campaign's three reports and then the host's withdrawal of ONLY that record's state, in that order, for the one placement
    let phases: Vec<(&str, &str)> = driven.fixed_log.iter().map(|entry| (entry.0.as_str(), entry.1.as_str())).collect();
    assert_eq!(
        phases,
        [
            ("0x41600000", "placed"),
            ("0x41600000", "placed"),
            ("0x41600000", "placed"),
            ("0x41500000", "removed")
        ],
        "{:?}",
        driven.fixed_log
    );
    assert_eq!(
        driven.roster_log.len(),
        2,
        "the placement itself stayed standing: only the boot and the insertion are roster events"
    );
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);

    // the prerequisites are not a pass when they were never reached: a city whose later question never read the state again (the run was clean), and a placement the host was never told a state for
    for (extra, why) in [
        (CITY_STATE.to_string(), "the withdrawn city was not asked about again"),
        (
            ", real_scan = true, fixed_candidate = true, city = 2, city_state_again = true, candidate_sequence = { 'need_fixed', 'buildings', 'test' }"
                .to_string(),
            "the host was never told a city state",
        ),
    ] {
        let error = error_of(drive(&healthy(&extra), &[("mode", "withdrawn_city_state")]));
        assert!(error.contains("UNREACHED") && error.contains("holds no such city"), "{why}: {error}");
    }
    // a failure of the run itself before any eligible lookup was answered is that failure, never an absent prerequisite
    let error = error_of(drive(
        &healthy(&format!("{extra}, engine_stop = {{ after = 1, marker = 'entity_candidate.callback' }}")),
        &[("mode", "withdrawn_city_state")],
    ));
    assert!(
        !error.contains("UNREACHED")
            && error.contains("stopped at [model-provider:entity_candidate.callback], not at [model-provider:entity_candidate.refused]"),
        "{error}"
    );
    let error = error_of(drive(
        &healthy(&format!("{extra}, no_candidate_key = true")),
        &[("mode", "withdrawn_city_state")],
    ));
    assert!(
        !error.contains("UNREACHED") && error.contains("request_faults=") && error.contains("is not a placement key"),
        "{error}"
    );
    let error = error_of(drive(
        &healthy(&format!("{extra}, bad_real_candidate = 'state'")),
        &[("mode", "withdrawn_city_state")],
    ));
    assert!(
        !error.contains("UNREACHED") && error.contains("reply_faults=1") && error.contains("candidate reply:"),
        "{error}"
    );
    let error = error_of(drive(
        &healthy(", fixture_api = true"),
        &[("profile", "original"), ("mode", "withdrawn_city_state")],
    ));
    assert!(error.contains("only mode=control"), "{error}");
}

#[test]
fn a_wrong_served_key_the_real_provider_could_answer_is_neither_credited_as_a_city_lookup_nor_withdrawn() {
    // the real provider holds a city state for a SECOND live placement and the candidate names it, although the host served the first: a request fault, no credit, nothing withdrawn
    let extra = format!("{CITY_STATE}, second_held = true, candidate_second = true, city_state_again = true");
    let driven = drive(&healthy(&extra), &[("mode", "withdrawn_city_state")]);
    let row = authority(&driven);
    assert_eq!(
        (
            row["city_state_joined"].clone(),
            row["city_state_withdrawn"].clone(),
            row["city_state_withdrawn_next"].clone()
        ),
        (Value::from(0), Value::from(0), Value::from("")),
        "{row}"
    );
    assert_eq!((row["candidate_altered"].clone(), row["altered"].clone()), (Value::from(0), Value::from(0)));
    assert!(row["request_faults"].as_i64().unwrap() >= 1, "{row}");
    assert!(
        driven.fixed_log.iter().all(|entry| entry.1 == "placed"),
        "nothing was removed: {:?}",
        driven.fixed_log
    );
    let error = error_of(driven);
    assert!(
        !error.contains("UNREACHED")
            && error.contains("candidate request: the candidate names the placement 2, which is not the entry the host last served (1)"),
        "{error}"
    );
}

#[test]
fn each_single_fault_of_the_city_state_grammar_is_counted_at_its_question_and_is_never_a_pass() {
    for (fault, expected) in [
        (
            "stray_fact",
            "states nothing but the position, the box minimum, the box maximum, the sub type and its phase",
        ),
        (
            "word_13",
            "states nothing but the position, the box minimum, the box maximum, the sub type and its phase",
        ),
        ("no_key", "names \"\", which is not a roster key"),
        ("bad_key", "names \"01\", which is not a roster key"),
    ] {
        let (driven, row) = scan_row(&format!("{CITY_STATE}, bad_city_state_request = '{fault}'"), "scan_real");
        assert!(driven.result.is_err(), "{fault}: {:?}", driven.result);
        assert!(row["request_faults"].as_i64().unwrap() >= 1, "{fault}: {row}");
        let first = row["first_fault"].as_str().unwrap();
        assert!(first.starts_with("scan request:") && first.contains(expected), "{fault}: {first}");
    }
    for (fault, expected) in [("echo", "acknowledged \"99\" for the placement 1"), ("state", "answered removed, not placed")] {
        let (driven, row) = scan_row(&format!("{CITY_STATE}, bad_real_fixed = '{fault}'"), "scan_real");
        assert!(driven.result.is_err(), "{fault}: {:?}", driven.result);
        assert!(row["reply_faults"].as_i64().unwrap() >= 1, "{fault}: {row}");
        let first = row["first_fault"].as_str().unwrap();
        assert!(first.starts_with("scan reply:") && first.contains(expected), "{fault}: {first}");
    }
}

// ---- THE HOST BUILDS A CITY'S BOUNDING BOX: the boot's questions and the boot-phase negative (EEDCS-49, D3, Task49) ----
//
// A building's link asks the host to build its city's box (phase 15): at BOOT, when the campaign's cities are made. The driver counts the questions in the window before the launch (`city_expand_before`), checks every
// reply and alters nothing; `withdrawn_box_state` is the host-side negative at that boot: at the first question whose pair joined a city state and a building state the provider holds, the host withdraws ONLY that
// building's fixed state (acknowledged `removed`) and the SAME question is delegated once, unmodified: the provider's named refusal stops the boot. These tests run the REAL driver against the scripted engine
// (the scripted provider answers the expansion from what the REAL roster holds: tests/model_target_point.rs pins the real provider's own answers and refusals).

const CITY_BOX: &str = ", real_scan = true, city_box = 2";

#[test]
fn the_cities_made_at_boot_ask_the_host_for_each_boxes_and_the_driver_counts_checks_and_alters_nothing() {
    let (driven, row) = scan_row(&format!("{CITY_BOX}, city_state = false"), "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    // two buildings linked at boot: two questions in the window before the launch, none after, both answered `expanded` with the city's key
    assert_eq!(
        (
            row["city_expand_before"].clone(),
            row["city_expand_after"].clone(),
            row["city_expand_total"].clone(),
            row["city_expand_answered"].clone()
        ),
        (Value::from(2), Value::from(0), Value::from(2), Value::from(2)),
        "{row}"
    );
    assert_eq!(
        (row["request_faults"].clone(), row["reply_faults"].clone()),
        (Value::from(0), Value::from(0)),
        "{row}"
    );
    assert_eq!(driven.expand_log.len(), 2, "{:?}", driven.expand_log);
    for (phase, result, name, identity) in &driven.expand_log {
        assert_eq!((phase.as_str(), result.as_str()), ("0x41700000", "expanded"));
        assert!(
            identity.starts_with(&format!("{name} ")),
            "the identity place is the city's key and the building's: {identity}"
        );
    }
    // the diagnostic counts of the other questions are the roster's own: the city's one state report at its insertion and one state report per building (the building's fixed state), no restatement of the city's box
    let phases: Vec<&str> = driven.fixed_log.iter().map(|entry| entry.0.as_str()).collect();
    assert_eq!(phases, ["0x41600000", "0x41500000", "0x41500000"], "{:?}", driven.fixed_log);
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);
}

#[test]
fn withdrawing_a_buildings_state_before_its_citys_box_is_built_makes_the_same_question_refused_by_name_and_stops_the_boot() {
    let driven = drive(&healthy(CITY_BOX), &[("mode", "withdrawn_box_state")]);
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    let row = authority(&driven);
    assert_eq!(row["completed"], false);
    assert_eq!(
        (row["boot_stop"].clone(), row["failure_marker"].clone()),
        (Value::from(true), Value::from("entity_scan.refused")),
        "{row}"
    );
    // the question joined a held city state and a held building state (eligible), the host's own withdrawal was acknowledged, and the real refusal is named
    assert_eq!(
        (
            row["box_eligible"].clone(),
            row["box_withdrawn"].clone(),
            row["box_withdrawn_cause"].clone(),
            row["city_expand_total"].clone(),
            row["city_expand_answered"].clone()
        ),
        (
            Value::from(1),
            Value::from(1),
            Value::from("city_box_building_state_withdrawn"),
            Value::from(1),
            Value::from(0)
        ),
        "{row}"
    );
    // the only fault is the provider's named refusal: no request was altered and no answer was forged; every physical fact after the stop is unknown, never read
    assert_eq!(
        (
            row["request_faults"].clone(),
            row["reply_faults"].clone(),
            row["first_fault"].clone(),
            row["physical_after_stop"].clone()
        ),
        (Value::from(0), Value::from(1), Value::from("scan reply: result refused"), Value::from(-1)),
        "{row}"
    );
    // the provider acknowledged the city's state and the building's, then the host's withdrawal of ONLY the building's, and was asked the question once, refused
    let phases: Vec<(&str, &str)> = driven.fixed_log.iter().map(|entry| (entry.0.as_str(), entry.1.as_str())).collect();
    assert_eq!(
        phases,
        [("0x41600000", "placed"), ("0x41500000", "placed"), ("0x41500000", "removed")],
        "{:?}",
        driven.fixed_log
    );
    assert_eq!(driven.expand_log.len(), 1, "{:?}", driven.expand_log);
    let (_, result, _, identity) = &driven.expand_log[0];
    assert_eq!(result, "refused");
    // the withdrawal named the very building the delegated question named, and the delegated question is the campaign's own
    assert_eq!(
        driven.fixed_log[2].3,
        identity.split(' ').nth(1).expect("the building's key"),
        "{:?}",
        driven.fixed_log
    );
    assert!(driven.delegation_broken.is_empty(), "{:?}", driven.delegation_broken);
    // the engine was stopped at boot: nothing after the boot was asked
    assert!(driven.launch.is_none(), "no weapon was launched after the stop at boot");
}

#[test]
fn a_boot_that_asks_no_city_expansion_is_unreached_for_the_boot_phase_negative_and_never_a_pass() {
    let error = error_of(drive(&healthy(", real_scan = true"), &[("mode", "withdrawn_box_state")]));
    assert!(
        error.contains("UNREACHED: withdrawn_box_state") && error.contains("no actor, time or geometry was searched for"),
        "{error}"
    );
}

#[test]
fn a_city_expansion_question_that_is_not_the_contracts_is_a_request_fault_and_is_never_followed_by_a_withdrawal() {
    for (fault, why) in [
        ("stray_fact", "a city expansion states nothing but its phase"),
        ("one_key", "which is not a city key, one space and a building key"),
        ("bad_key", "which is not a city key, one space and a building key"),
    ] {
        let extra = format!("{CITY_BOX}, bad_expand_request = '{fault}'");
        let error = error_of(drive(&healthy(&extra), &[("mode", "withdrawn_box_state")]));
        assert!(
            error.contains("nothing was withdrawn because the boot failed first") && error.contains(why),
            "{fault}: {error}"
        );
    }
}

#[test]
fn a_city_expansion_reply_that_is_not_the_contracts_is_a_reply_fault_of_the_healthy_boot() {
    for (fault, why) in [
        // the shared C's own check stops the engine at boot for a wrong state or another city's key (the scripted engine applies it): the driver never sees a boot that went on
        ("state", "[model-provider:entity_scan.state]"),
        ("echo", "[model-provider:entity_scan.state]"),
        // what the shared C would also refuse, the driver's own check of the reply names: the box is not six patterns, or a word of it is not a finite number
        (
            "short",
            "reply_faults=2): scan reply: a city expansion answered a box that is not six 0x%08x patterns",
        ),
        (
            "infinite",
            "reply_faults=2): scan reply: a city expansion answered the box word 0x7f800000, which is not a finite number",
        ),
    ] {
        let extra = format!("{CITY_BOX}, bad_real_expand = '{fault}'");
        let result = drive(&healthy(&extra), &[("mode", "control")]).result;
        let error = result.expect_err("a malformed reply is a failure of the healthy run");
        assert!(error.contains(why), "{fault}: {error}");
    }
}

// The pair a city expansion names must be the CURRENT city and a building that is listed under THAT city's list. The driver joins the sector placement (through the key its list is named by, which a relink of the
// placement keeps) to the list the served building is listed under BEFORE it counts the question eligible or sends the host's own withdrawal. A pair of two live cities with their own acknowledged states and
// a building of the OTHER city is a request fault: nothing is eligible, nothing is withdrawn or restated, and the boot's verdict is a failure.

fn assert_no_withdrawal_of_a_mismatched_pair(layout: &str) {
    let extra = format!("{CITY_BOX}, city_box_layout = '{layout}'");
    let driven = drive(&healthy(&extra), &[("mode", "withdrawn_box_state")]);
    let row = authority(&driven);
    assert_eq!(
        (row["box_eligible"].clone(), row["box_withdrawn"].clone(), row["box_withdrawn_cause"].clone()),
        (Value::from(0), Value::from(-2), Value::from("")),
        "{layout}: {row}"
    );
    assert!(row["request_faults"].as_i64().unwrap() >= 1, "{layout}: {row}");
    assert!(row["first_fault"].as_str().unwrap().contains("is not listed under the city"), "{layout}: {row}");
    // the provider acknowledged only states: no building's state was withdrawn and no state was restated
    assert!(
        driven.fixed_log.iter().all(|entry| entry.1 == "placed"),
        "{layout}: nothing was withdrawn: {:?}",
        driven.fixed_log
    );
    let error = error_of(driven);
    assert!(
        !error.contains("UNREACHED") && error.contains("nothing was withdrawn because the boot failed first") && error.contains("is not listed under the city"),
        "{layout}: {error}"
    );
}

#[test]
fn a_building_of_the_other_city_is_never_eligible_and_no_state_is_withdrawn_for_it() {
    assert_no_withdrawal_of_a_mismatched_pair("cross");
}

#[test]
fn a_relinked_city_is_still_not_the_city_of_a_building_listed_under_another() {
    assert_no_withdrawal_of_a_mismatched_pair("cross_relinked");
}

#[test]
fn the_current_pair_of_a_relinked_or_an_empty_relinked_city_is_eligible_and_the_withdrawal_is_credited() {
    for layout in ["relink", "empty_relink"] {
        let extra = format!("{CITY_BOX}, city_box_layout = '{layout}'");
        let driven = drive(&healthy(&extra), &[("mode", "withdrawn_box_state")]);
        assert!(driven.result.is_ok(), "{layout}: {:?}\n{}", driven.result, driven.rows);
        let row = authority(&driven);
        assert_eq!(
            (
                row["box_eligible"].clone(),
                row["box_withdrawn"].clone(),
                row["box_withdrawn_cause"].clone(),
                row["request_faults"].clone(),
                row["reply_faults"].clone()
            ),
            (
                Value::from(1),
                Value::from(1),
                Value::from("city_box_building_state_withdrawn"),
                Value::from(0),
                Value::from(1)
            ),
            "{layout}: {row}"
        );
        // exactly the building's state was withdrawn, once, and the question was delegated once
        let removed: Vec<_> = driven.fixed_log.iter().filter(|entry| entry.1 == "removed").collect();
        assert_eq!(removed.len(), 1, "{layout}: {:?}", driven.fixed_log);
        assert_eq!(driven.expand_log.len(), 1, "{layout}: {:?}", driven.expand_log);
        assert_eq!(driven.expand_log[0].1, "refused", "{layout}");
    }
}

#[test]
fn a_boot_stop_at_the_expected_generic_marker_with_nothing_withdrawn_is_a_failure_and_never_unreached() {
    let extra = format!("{CITY_BOX}, city_box_layout = 'unrelated_refusal'");
    let driven = drive(&healthy(&extra), &[("mode", "withdrawn_box_state")]);
    let row = authority(&driven);
    assert_eq!(
        (
            row["boot_stop"].clone(),
            row["failure_marker"].clone(),
            row["box_eligible"].clone(),
            row["box_withdrawn"].clone(),
            row["box_withdrawn_cause"].clone(),
            row["physical_after_stop"].clone()
        ),
        (
            Value::from(true),
            Value::from("entity_scan.refused"),
            Value::from(0),
            Value::from(0),
            Value::from(""),
            Value::from(-1)
        ),
        "{row}"
    );
    let error = error_of(driven);
    assert!(
        !error.contains("UNREACHED") && error.contains("before a city expansion was withdrawn") && error.contains("an unrelated refusal at boot"),
        "{error}"
    );
}

#[test]
fn a_city_expansion_reply_that_is_not_exactly_six_patterns_is_a_reply_fault_though_it_has_the_length_of_one() {
    for fault in ["commas", "signed", "blank"] {
        let extra = format!("{CITY_BOX}, bad_real_expand = '{fault}'");
        let driven = drive(&healthy(&extra), &[("mode", "control")]);
        let row = authority(&driven);
        // never credited as an answered expansion
        assert_eq!(row["city_expand_answered"], 0, "{fault}: {row}");
        let error = error_of(driven);
        assert!(
            error.contains("scan reply: a city expansion answered a box that is not six 0x%08x patterns"),
            "{fault}: {error}"
        );
    }
}

#[test]
fn a_city_expansion_reply_of_six_exact_patterns_is_credited_with_a_signed_zero_and_a_subnormal() {
    let extra = format!("{CITY_BOX}, expanded_p = '0x80000000,0x00000001,0xc9742400,0x00000000,0x00000001,0x49742400'");
    let (driven, row) = scan_row(&extra, "control");
    assert!(driven.result.is_ok(), "{:?}\n{}", driven.result, driven.rows);
    assert_eq!(
        (row["city_expand_answered"].clone(), row["reply_faults"].clone()),
        (Value::from(2), Value::from(0)),
        "{row}"
    );
}
