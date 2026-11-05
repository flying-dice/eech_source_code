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
POSE_LOG = {}
MOVES, RETIRED, DISPLACED, MOVE_LOG = {}, {}, {}, {}
VECTORS_ASKED, VECTORS_ALTERED, VECTORS_SEEN = 0, false, nil
MQ_CALLS = {}
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
		-- the REAL enumeration answers, by the phase the campaign reports: the first sector (0, 0) at the start, an offer for every served entry that is not excluded, the next entry after a miss, the end of the
		-- scan when no entry is left or after a hit
		if scenario.real_scan_error then error ("the real provider refuses", 0) end
		local words = {}
		for token in string.gmatch (key, "[^,]+") do words [#words + 1] = token end
		local phase, excluded, hit = words [12], words [15] ~= ZERO, words [16] ~= ZERO
		if phase == ZERO then answer = scenario.scan_offers_nothing and { result = "done_none" } or { result = "first", p = ZERO .. "," .. ZERO }
		elseif phase == "0x3f800000" then answer = { result = excluded and "next" or "offer" }
		elseif phase == "0x40000000" then answer = { result = "done_none" }
		else answer = { result = hit and "done_hit" or "next" } end
		local bad = scenario.bad_real_scan
		if bad == "state" then answer.result = "perhaps" end
		if bad == "p_on_next" and answer.result ~= "first" then answer.p = ZERO .. "," .. ZERO end
		if bad == "name" then answer.name = "SECTOR" end
		if bad == "first_one_pattern" and answer.result == "first" then answer.p = ZERO end
		SCAN_LOG [#SCAN_LOG + 1] = { phase = phase, result = answer.result, key = key }
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
			local altering = { candidate_skip = true, withheld_candidate = true, bad_candidate_state = true, bad_candidate_identity = true }
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
			local function candidate_dialogue ()
				if scenario.no_candidate_questions then return true end
				local mobile = not scenario.fixed_candidate
				local scene = mobile and 7 or 0
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
					elseif result == "need_placement" then
						if words [44] ~= ZERO or scene == 0 then stop ("entity_candidate.state", "placement asked of a candidate that cannot need it") end
						words [44], words [23] = "0x3f800000", "0x3f800000"
						if mobile then pose_observe (1) end
					elseif result == "need_normal" then
						if words [24] ~= ZERO or words [44] == ZERO or words [23] == ZERO or mobile then stop ("entity_candidate.state", "normal asked of a candidate that cannot need it") end
						words [24] = "0x3f800000"
					elseif result == "buildings" then
						if words [9] == ZERO then stop ("entity_candidate.state", "buildings named for a candidate that is not a city") end
						return false
					else
						local count = 0
						if type (answer.p) == "string" then for _ in string.gmatch (answer.p, "[^,]+") do count = count + 1 end end
						if count ~= 12 and count ~= 16 then stop ("entity_candidate.pose", "pose of " .. count) end
						if scene == 0 then stop ("entity_candidate.state", "a narrow phase for a candidate that has no scene") end
						return true
					end
				end
				stop ("entity_candidate.state", "the dialogue did not end")
			end
			-- the campaign's driver of the enumeration (mb_hostscan.h) as the shared C applies it: it asks the host, serves the sector (0, 0) with ONE entry, offers it, reports the result and ends when the host does. The
			-- legality of each state against the phase and the cursor's grammar are the exchange's rules; the stop text is the real wrapper. `offer` runs the candidate dialogue and the narrow phase of ONE entry
			local scan_altering = { scan_none = true, withheld_scan = true, bad_scan_state = true, bad_scan_cursor = true }
			local function scan_walk (offer)
				if scenario.no_scan_questions then offer () return end
				local words = {}
				for index = 1, 16 do words [index] = ZERO end
				words [1], words [2], words [3] = OLD_W [1], OLD_W [2], OLD_W [3]
				words [4], words [5], words [6] = NEW_W [1], NEW_W [2], NEW_W [3]
				words [7], words [10], words [11] = "0x42c80000", "0x43c78000", "0x43c78000"
				local PHASES = { begin = ZERO, entry = "0x3f800000", ["end"] = "0x40000000", resolved = "0x40400000" }
				local function ask (phase, excluded, hit)
					words [12], words [15], words [16] = PHASES [phase], excluded and "0x3f800000" or ZERO, hit and "0x3f800000" or ZERO
					local key = table.concat (words, ",")
					if scenario.bad_scan_request == "count" then key = string.match (key, "^(.*),[^,]+$") end
					local ok, answer = ask_model ("entity_scan", 0, nil, nil, nil, key, not scan_altering [MODE])
					if not ok then stop ("entity_scan.callback", tostring (answer)) end
					if type (answer) ~= "table" or not ({ first = true, next = true, offer = true, done_none = true, done_hit = true }) [answer.result] then stop ("entity_scan.answer", "result") end
					return answer
				end
				local function allowed (answer, ...)
					for _, state in ipairs ({ ... }) do if answer.result == state then return end end
					stop ("entity_scan.state", "state " .. answer.result .. " is not allowed here")
				end
				local answer = ask ("begin", false, false)
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
				answer = ask ("entry", false, false)
				allowed (answer, "offer")
				last_hit = false
				offer ()
				answer = ask ("resolved", false, last_hit)
				allowed (answer, last_hit and "done_hit" or "next")
				if answer.result == "done_hit" then return end
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
				for _, index in ipairs (scenario.steps or {}) do
					local ground = scenario.ground or 10
					local step = { host_id = "eech-world:" .. index, mode = "move", terrain = { elevation = ground, forest = false, face_normal = { 0, 1, 0 } }, map_max = { 1000, scenario.ceiling or 2000, 1000 } }
					config.move_vehicle (step)
					-- the engine's own step table is never mutated by the host-movement wrapper
					if step.terrain.elevation ~= ground then CALLER_MUTATED = true end
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
					for k = 0, 2 do
						-- the production caller's questions about the target: the hosted module asks the host, the original asks nothing
						if not scenario.query_original then scan_walk (function () candidate_dialogue () end) end
						local a, b = segment_text (k)
						if scenario.query_segment_changed and k == 2 then a = "0x41000000,0x41000001,0x41000003" end
						local head = "wpn-moving-query-result seg=" .. k .. " hit=" .. ((scenario.query_hits or { 1, 0, 1 }) [k + 1]) .. " a=" .. a .. " b=" .. b
						if (scenario.query_hits or { 1, 0, 1 }) [k + 1] == 1 then
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
    /// (phase pattern, result) of every scan question the REAL provider answered
    scan_log: Vec<(String, String)>,
    /// (stage, entity type, alive, result, key) of every pose observation the REAL provider answered
    pose_log: Vec<(i64, i64, i64, String, String)>,
    /// (identity, mode, terrain elevation or -1, after the launch) of every step the host's movement callback was given
    move_log: Vec<(String, String, f64, bool)>,
    /// the `vectors` aim requests the scripted campaign made, and whether any was not forwarded as the very same unchanged request
    vectors: (i64, bool),
    /// the controlled query surface's calls, in order, each with the simulated time it was made at
    query_calls: Vec<String>,
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
    for mode in ["scan_real", "scan_none", "withheld_scan", "bad_scan_state", "bad_scan_cursor"] {
        assert!(
            error_of(drive("", &[("profile", "original"), ("mode", mode)])).contains("only mode=control"),
            "{mode}"
        );
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
