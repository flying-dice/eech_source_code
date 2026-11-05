-- TEST-ONLY host-authority and refusal witness of the production ENTITY_COLLISION narrow-phase handoff (EEDCS-49, D3). Needs a DLL built with the `weapon-fixture` feature (the controlled launch of
-- lua/weapon-case.lua) and an eech-world with `host.model` and `host.create_exclusive`.
--
-- Boundary under test (the product path, not a private test of Rust or C): host-owned Lua -> the public native module -> the host's MODEL provider (`host.model`) -> back into a REAL weapon
-- launched through the campaign's own launch call. The hosted weapon flight asks the campaign for the entity-collision lookup (wn_move.c CAMPAIGN_STEP_LOOKUP_COLLISION), which calls
-- get_line_of_sight_collision_entity; for each collision CANDIDATE whose broad phase (still Core: the sector scan and a box of radius + 10 m) passes, the hosted narrow phase asks the HOST
-- `entity_collision` through the model provider (mb_hostentity.h). The module has no model transform or intersection behind that question.
--
-- WHAT THE QUESTION AND THE ANSWER ARE (the corrected shape, mb_hostmodel.h ENTITY_COLLISION): the request's text place `key` holds 18 (a stationary candidate) or 22 (a moving one)
-- comma-separated float bit patterns `0x%08x`: the segment's OLD end (3) and NEW end (3), then the candidate's pose. The caller's intercept point and normal are OUTPUTS and are not sent. The
-- answer is { result = "hit" | "miss", mask = 0..63, p = six bit patterns, name = the scene's compiled name }: `mask` says which output components the original assigned (bits 0 to 2 the
-- point, 3 to 5 the normal; each vector whole, a point implies a normal, a hit is 63), `p` is the point then the normal (zero for an unassigned component). The campaign writes only the
-- assigned components after checking the whole reply.
--
-- BRANCH PRECONDITIONS, from source (nothing here is a guess about a distance):
--   1. the weapon step asks the entity lookup only when the weapon is a server weapon with in-flight collision checking AND (the step is at high collision precision OR the weapon has no target)
--      (wn_flight.c). The launch links the case target for the launch, so the precision path is needed. The existing protocol's VIEW_WEAPON answer is the same configured input as in
--      tree-authority.lua (below): `precision_high` must rise, and the fixture counter `contact_any_tests` must rise, or the actual entity caller was not reached (UNREACHED).
--   2. get_line_of_sight_collision_entity excludes only the launcher (the weapon's los_target is NULL) and visits the sector window of the segment widened by 150 m; a candidate gets a
--      narrow-phase question only if its type is collidable (a vehicle or an aircraft is a mobile candidate, a structure a fixed one; a PERSON has neither flag) and the segment's box meets
--      the candidate's box of radius + 10 m. So the case is the NEAREST non-person enemy of any launcher that has an unguided ballistic weapon in stock: the pair with the smallest distance,
--      so that the target itself is a broad-phase candidate of the weapon's first step. A war in which no candidate ever passes the broad phase makes ZERO questions: that is UNREACHED (the
--      broad phase and the sector order are still the campaign's), never authority.
--
-- TWO PROFILES, named by `profile=`:
--   hosted (default)   the module as shipped: built with `weapon-fixture` and NOT `collision-fixture`.
--   original           the explicitly original characterisation: built with `weapon-fixture` AND `collision-fixture`, whose narrow phase is the unmoved original (EECH_CAMPAIGN_HOST_ENTITY is not
--                      defined). Only mode=control. It is the capability RED of the host route: the fixture API is PRESENT, the weapon step's entity lookup was made (`contact_any_tests` rose,
--                      so the actual entity-collision caller was reached) and the host was asked no entity_collision question. That is reported as `CAPABILITY RED` and the process FAILS on
--                      purpose (the expected red). A module without the fixture API, or a run in which the weapon step made no entity test, is `UNREACHED`: the call this RED is about never
--                      happened. The counter cannot show that the original narrow phase itself ran (it counts the caller's lookup); the row says how many lookups hit.
--
-- One bounded fresh war per process. The provider is the host's own (host.model), delegated to unchanged and counted, until the case weapon is launched ("armed"); from then on the chosen mode
-- may answer an entity_collision question differently. Every other operation is ALWAYS the host's own answer, byte for byte:
--
--   control           nothing is altered: the healthy same-DLL control. Self-asserted from what the provider SEES after the launch: at least one entity_collision question (else UNREACHED),
--                     each request is 18 or 22 well-formed patterns, each real reply is well formed (result hit or miss, a mask inside the domain with whole vectors and a point only with a normal,
--                     a hit with mask 63, zero words for what the mask does not assign, a name), and the questions relate to the fixture's own counters (the lookups the weapon step made,
--                     the hits among them).
--   forced_hit        every armed question is answered HIT with mask 63: the point is the request's own OLD end and the normal is up. The real answer is still asked of the host (its name is the
--                     echo the campaign requires; its result is discarded). A VALID changed answer: the weapon step's actual hit count (contact_any_hits) must rise, and the cohort's PHYSICAL
--                     signature (type, code, steps, position, velocity, lifetime of each case weapon, not its ordinal) is recorded for comparison with the control.
--   forced_miss       every armed question is answered MISS with mask 56 (the normal assigned, the point not): a VALID accepted miss that writes only the normal. No hit may then reach the
--                     weapon step (contact_any_hits does not rise). If the control had no hits this is valid MISS coverage, not a measured changed physical outcome, and the row says so.
--   withheld_entity   the provider raises on the first armed question:                                  [model-provider:entity_collision.callback]
--   bad_write_mask    a HIT answered with mask 56 (a hit has assigned both vectors):                     [model-provider:entity_collision.write_mask]
--   bad_answer        a hit answered with mask 64 (outside the six components):                         [model-provider:entity_collision.answer]
--
-- THE CONTROLLED VIEW INPUT. See tree-authority.lua: the existing protocol's VIEW_WEAPON lookup answer (mb_hostaim.h, answered r0 by wn_move.c, f[7] in Lua) is set to 1 by an aim delegate over the
-- unmodified host.aim on every armed weapon_step request, hosted and Original alike. It is a CONFIGURED, GLOBAL-while-armed diagnostic input (the protocol carries no weapon identity, so natural
-- weapons are affected too), not a camera, not a per-weapon identity and not an entity call; the entity lookup, the questions and the counters stay measured. A run in which it never reached a
-- weapon step, or the step never took the high-precision path, is UNREACHED.
--
-- WHAT COUNTS AS THE ENGINE STOPPING: only the first line of the failure, anchored: `runtime error: eech: EECH fatal error: host model: [model-provider:<marker>] `. A Lua error that merely
-- mentions a marker, a witness failure, a failure without that wrapper and a different marker are failures of the witness, never proof. After the stop NOTHING is asked of the engine again (no
-- census, report, query or restoration), counters that were not read stay unknown (-1) and no result of the campaign continuing is claimed.
--
-- WHAT COUNTS AS A COMPLETED CASE and the evidence file: as tree-authority.lua (the launch report joined with the logged births, a frozen cohort, one complete terminal per member before the
-- settle cap, terminals of weapons outside the cohort declared foreign; the provider's questions and the fixture's counters are global to the war and are NOT attributed to the case weapons).
-- The evidence file is created exclusively with `host.create_exclusive` (a checked close). Exit 0 is the verdict.
--
-- host.args: root, mode (default control), profile (default hosted), warmup (simulated seconds before the launch, default 30), settle (cap on the simulated seconds after it, default 120),
--            frame_ms (default 100), out (JSON Lines path, must not exist)

io.stdout:setvbuf ("no")

local NAME = "entity-authority"
local function refuse (message) error (NAME .. ": " .. message, 0) end

local huge = math.huge
local function finite (v) return type (v) == "number" and v == v and v ~= huge and v ~= -huge end

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

local args = host.args
local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local mode = args.mode or "control"
local profile = args.profile or "hosted"
if profile ~= "hosted" and profile ~= "original" then refuse ("profile must be hosted or original, got '" .. tostring (profile) .. "'") end
local warmup = tonumber (args.warmup or "30")
local settle = tonumber (args.settle or "120")
local frame_ms = tonumber (args.frame_ms or "100")
if not (finite (warmup) and finite (settle) and finite (frame_ms)) then refuse ("warmup, settle and frame_ms must be finite numbers") end
if not (warmup >= 1 and warmup <= 600 and settle >= 1 and settle <= 1200 and frame_ms >= 10 and frame_ms <= 1000) then refuse ("warmup, settle or frame_ms is out of range") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local seed = 1
-- the most frames the whole run can need is far below this: the simulated clock must advance on every frame, and the budget is four times what warmup and settle ask for, plus a margin
local frame_budget = 4 * math.ceil ((warmup + settle) * 1000 / frame_ms) + 200


-- ---------------------------------------------------------------------------------------------------------------- the modes
local ZERO = "0x00000000"
local UP = ZERO .. ",0x3f800000," .. ZERO
local NO_POINT = ZERO .. "," .. ZERO .. "," .. ZERO
-- the first three patterns of the request's text: the segment's OLD end
local function old_end (parsed) return parsed.words [1] .. "," .. parsed.words [2] .. "," .. parsed.words [3] end
local MODES = {
	control = { altering = false },
	forced_hit = {
		-- HIT, mask 63, at the segment's OLD end with an up normal: complete write information for a hit
		answer = function (parsed, real) return { result = "hit", mask = 63, name = real.name, p = old_end (parsed) .. "," .. UP } end,
		altering = true, expect = "hit",
	},
	forced_miss = {
		-- MISS, mask 56: the normal assigned (up), the point not: an accepted partial-write miss
		answer = function (_, real) return { result = "miss", mask = 56, name = real.name, p = NO_POINT .. "," .. UP } end,
		altering = true, expect = "miss",
	},
	withheld_entity = { answer = function () error ("the host withholds its entity answer", 0) end, marker = "entity_collision.callback" },
	bad_write_mask = {
		answer = function (_, real) return { result = "hit", mask = 56, name = real.name, p = NO_POINT .. "," .. UP } end,
		marker = "entity_collision.write_mask",
	},
	bad_answer = {
		answer = function (parsed, real) return { result = "hit", mask = 64, name = real.name, p = old_end (parsed) .. "," .. UP } end,
		marker = "entity_collision.answer",
	},
}
local plan = MODES [mode]
if plan == nil then refuse ("unknown mode '" .. tostring (mode) .. "'") end
if profile == "original" and mode ~= "control" then refuse ("profile=original runs only mode=control: no entity_collision question reaches the provider to alter") end

-- ---------------------------------------------------------------------------------------------------------------- the evidence file: created exclusively, every write, flush and close checked, closed on every path
local create = host.create_exclusive
if type (create) ~= "function" then refuse ("this host has no host.create_exclusive: the evidence cannot be created exclusively with a checked close, and the witness does not fall back to a truncating open") end
local out, create_why = create (out_path)
if not out then refuse ("OUTPUT FAILURE: creating " .. out_path .. " failed: " .. tostring (create_why) .. " (the evidence is created exclusively and never overwritten; nothing was run)") end

local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
local function json_string (s)
	return '"' .. (string.gsub (s, '[%c"\\\128-\255]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)) .. '"'
end
local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		local value
		if type (v) == "string" then value = json_string (v)
		elseif type (v) == "boolean" then value = v and "true" or "false"
		else value = tostring (v) end
		parts [#parts + 1] = json_string (k) .. ":" .. value
	end
	table.sort (parts)
	local ok, err = out:write ("{" .. table.concat (parts, ",") .. "}\n")
	if not ok then error (NAME .. ": OUTPUT FAILURE: writing " .. out_path .. " failed: " .. tostring (err), 0) end
	ok, err = out:flush ()
	if not ok then error (NAME .. ": OUTPUT FAILURE: flushing " .. out_path .. " failed: " .. tostring (err), 0) end
end
-- a failure of the witness itself: recorded if the evidence can still be written, then raised with the prefix the engine-failure handlers never treat as the engine's
local function hard (message)
	pcall (emit, { event = "witness_failure", reason = message })
	error (NAME .. ": " .. message, 0)
end
local function first_line (text) return (string.match (tostring (text), "[^\r\n]+") or tostring (text)) end

-- ---------------------------------------------------------------------------------------------------------------- the grammar of the existing C emitters (eech_wpn_fixture.c)
-- a whole number as the C side prints it (%d / %llu): optional minus, digits, nothing else (no fraction, exponent, nan or inf)
local function whole (text)
	if type (text) ~= "string" or not string.match (text, "^%-?%d+$") then return nil end
	return tonumber (text)
end
-- a finite number as %.9g prints it: digits with an optional fraction and an optional signed exponent
local function printed_number (text)
	if type (text) ~= "string" then return nil end
	if not (string.match (text, "^%-?%d+%.?%d*$") or string.match (text, "^%-?%d+%.?%d*e[%+%-]%d+$")) then return nil end
	local value = tonumber (text)
	if value == nil or value ~= value or value == huge or value == -huge then return nil end
	return value
end
-- the first problem of a wpn-case-launch report against the emitter's schema and domains, or nil: launcher is a name, weapon / target links / counts are whole numbers (a link is 0 or 1), creates
-- is a positive whole number, restored is -1 (the launcher's link was already the target: nothing changed, so the link before and after are the same) or 1 (restored); 0 is a failed restoration
-- and nothing else is in the schema
local function launch_report_problem (report, line, weapon)
	local function bad (name) return "the launch report does not follow the wpn-case-launch schema (" .. name .. "): " .. line end
	if type (report.launcher) ~= "string" or report.launcher == "" then return bad ("launcher") end
	local number = {}
	for _, name in ipairs { "weapon", "target_link_before", "target_link", "count_before", "count_after", "births", "creates", "restored" } do
		number [name] = whole (report [name])
		if number [name] == nil then
			return report [name] == nil and bad ("missing " .. name) or ("the launch report field " .. name .. "=" .. tostring (report [name]) .. " is not a whole number: " .. line)
		end
	end
	if number.weapon ~= weapon then return "the launch report says weapon=" .. number.weapon .. " but weapon " .. weapon .. " was launched: " .. line end
	for _, name in ipairs { "target_link_before", "target_link" } do
		if number [name] ~= 0 and number [name] ~= 1 then return "the launch report field " .. name .. "=" .. number [name] .. " is not 0 or 1: " .. line end
	end
	if number.count_before < 0 or number.count_after < 0 or number.births < 0 then return "the launch report has a negative count: " .. line end
	if number.count_after > number.count_before then return "the launch report has count_after=" .. number.count_after .. " above count_before=" .. number.count_before .. ": " .. line end
	if number.creates < 1 then return "the launch created no weapon (creates=" .. number.creates .. "): " .. line end
	if number.restored == 0 then return "the launch could not restore the launcher's temporary target link (restored=0): " .. line end
	if number.restored ~= -1 and number.restored ~= 1 then return "the launch report field restored=" .. number.restored .. " is not -1 or 1 (restored=0 is a failure, nothing else is in the schema): " .. line end
	if number.restored == -1 and number.target_link_before ~= number.target_link then return "restored=-1 means the link was kept, but the link changed from " .. number.target_link_before .. " to " .. number.target_link .. ": " .. line end
	return nil
end
-- the first problem of a weapon's terminal fact, or nil: every field the emitter writes for a registered weapon is present and in its grammar
local function terminal_problem (e)
	if whole (e.w) == nil or whole (e.w) < 0 then return "w" end
	if type (e.type) ~= "string" or e.type == "" then return "type" end
	if type (e.code) ~= "string" or e.code == "" or e.code == "invalid" then return "code" end
	local steps = whole (e.steps)
	if steps == nil or steps < 0 then return "steps" end
	local x, y, z = string.match (type (e.pos) == "string" and e.pos or "", "^([^,]+),([^,]+),([^,]+)$")
	if printed_number (x) == nil or printed_number (y) == nil or printed_number (z) == nil then return "pos" end
	if printed_number (e.velocity) == nil then return "velocity" end
	if printed_number (e.lifetime) == nil then return "lifetime" end
	return nil
end


-- ---------------------------------------------------------------------------------------------------------------- the engine's own stop, recognised only by its anchored first line
local FATAL = "^runtime error: eech: EECH fatal error: host model: %[model%-provider:([%w_%.]+)%] "
local function engine_marker (err) return string.match (first_line (err), FATAL) end

-- ---------------------------------------------------------------------------------------------------------------- the host's model provider, counted, altered only when armed
local armed = false
local reference -- the host's own model provider (host.model), resolved inside the owned close scope (main)
local before, after = { entity = 0, other = 0 }, { entity = 0, other = 0 }
local altered = 0
-- what the provider SEES after the launch (the whole war's questions: the provider carries no weapon identity)
local seen = { stationary = 0, moving = 0, real_hit = 0, real_miss = 0, real_mask_63 = 0, real_mask_56 = 0, real_mask_other = 0, answered_hit = 0, answered_miss = 0 }
local faults = { request = 0, reply = 0 }
local first_fault -- the first malformed request or real reply, kept for the evidence

local PATTERN = "^0x%x%x%x%x%x%x%x%x$"
local function words_of_text (text, count)
	local words = {}
	for token in string.gmatch (text, "[^,]+") do
		if not string.match (token, PATTERN) then return nil end
		words [#words + 1] = token
	end
	if (count and #words ~= count) or table.concat (words, ",") ~= text then return nil end
	return words
end
-- the request as the corrected contract states it: scene >= 1 and nothing but the text place, which holds 18 (stationary) or 22 (moving) float patterns
local function parse_request (scene, depth, kind, definition, key)
	if type (scene) ~= "number" or scene < 1 then return nil, "scene " .. tostring (scene) end
	if depth ~= nil or kind ~= nil or definition ~= nil then return nil, "an entity question carries only a scene and the pose text" end
	if type (key) ~= "string" then return nil, "no pose text" end
	local words = words_of_text (key)
	if words == nil then return nil, "the pose text is not comma separated 0x%08x patterns" end
	if #words ~= 18 and #words ~= 22 then return nil, "the pose has " .. #words .. " floats, not 18 or 22" end
	return { scene = scene, words = words, moving = #words == 22 }
end
-- the first problem of a REAL reply against the contract, or nil: result, mask domain, whole vectors, a point only with a normal, a hit is 63, zero words for what the mask does not assign, a name
local function reply_problem (answer)
	if type (answer) ~= "table" then return "the reply is not a table" end
	if answer.result ~= "hit" and answer.result ~= "miss" then return "result " .. tostring (answer.result) end
	local mask = answer.mask
	if not finite (mask) or mask ~= math.floor (mask) or mask < 0 or mask > 63 then return "mask " .. tostring (mask) end
	if type (answer.name) ~= "string" or answer.name == "" then return "no scene name" end
	if type (answer.p) ~= "string" then return "no words" end
	local words = words_of_text (answer.p, 6)
	if words == nil then return "the words are not six 0x%08x patterns" end
	local point, normal = mask % 8, math.floor (mask / 8)
	if (point ~= 0 and point ~= 7) or (normal ~= 0 and normal ~= 7) then return "mask " .. mask .. " assigns a vector in part" end
	if point ~= 0 and normal == 0 then return "mask " .. mask .. " assigns the point without the normal" end
	if answer.result == "hit" and mask ~= 63 then return "a hit with mask " .. mask end
	for index = 1, 6 do
		local assigned = (index <= 3 and point ~= 0) or (index > 3 and normal ~= 0)
		if not assigned and words [index] ~= ZERO then return "word " .. index .. " is not assigned by mask " .. mask .. " but is not zero" end
	end
	return nil
end
local function model (op, scene, depth, kind, definition, key)
	local counts = armed and after or before
	if op == "entity_collision" then counts.entity = counts.entity + 1 else counts.other = counts.other + 1 end
	-- every other operation, and every question before the launch, is the host's own answer, returned unchanged
	if op ~= "entity_collision" or not armed then return reference (op, scene, depth, kind, definition, key) end
	local parsed, why = parse_request (scene, depth, kind, definition, key)
	if parsed then
		if parsed.moving then seen.moving = seen.moving + 1 else seen.stationary = seen.stationary + 1 end
	else
		faults.request = faults.request + 1
		first_fault = first_fault or ("request: " .. why)
	end
	local real = reference (op, scene, depth, kind, definition, key)
	local problem = reply_problem (real)
	if problem then
		faults.reply = faults.reply + 1
		first_fault = first_fault or ("reply: " .. problem)
	else
		if real.result == "hit" then seen.real_hit = seen.real_hit + 1 else seen.real_miss = seen.real_miss + 1 end
		if real.mask == 63 then seen.real_mask_63 = seen.real_mask_63 + 1 elseif real.mask == 56 then seen.real_mask_56 = seen.real_mask_56 + 1 else seen.real_mask_other = seen.real_mask_other + 1 end
	end
	if plan.answer == nil then return real end
	-- an altered answer is formed only from a well-formed question and real reply; otherwise the host's own answer is returned unaltered and the fault (counted above) fails the verdict. A witness failure is NOT raised
	-- here: it would cross the engine's callback boundary and come back carrying the callback marker, which the refusal modes credit as the host withholding an answer
	if parsed == nil or problem then return real end
	altered = altered + 1
	local answer = plan.answer (parsed, real)
	if type (answer) == "table" then
		if answer.result == "hit" then seen.answered_hit = seen.answered_hit + 1 elseif answer.result == "miss" then seen.answered_miss = seen.answered_miss + 1 end
	end
	return answer
end

-- ---------------------------------------------------------------------------------------------------------------- the controlled view input (an aim delegate over the unmodified host.aim)
local VIEW_WEAPON = 1 -- mb_hostaim.h CAMPAIGN_STEP_LOOKUP_VIEW_WEAPON; its answer r0 is f[7] in Lua (f[6..11] hold r0..r5)
local view = { steps_seen = 0, answers_seen = 0, applied = 0, already_set = 0 }
local base_aim -- the host's own aim function, resolved inside the owned close scope (main)
-- a copy of one lookup answer with the sixth word set: kind, a..d and all twelve floats are copied, nothing is shared with the caller's table
local function viewed_answer (answer)
	if type (answer.f) ~= "table" or #answer.f ~= 12 then hard ("a VIEW_WEAPON answer does not carry twelve result words, so the controlled view cannot be applied exactly") end
	local copy = { kind = answer.kind, a = answer.a, b = answer.b, c = answer.c, d = answer.d, f = {} }
	for index = 1, 12 do copy.f [index] = answer.f [index] end
	if copy.f [7] ~= 0 then view.already_set = view.already_set + 1 end
	copy.f [7] = 1
	return copy
end
local function aim (request)
	if armed and type (request) == "table" and request.mode == "weapon_step" then
		view.steps_seen = view.steps_seen + 1
		local answers = request.answers
		if type (answers) == "table" then
			local changed, replaced = false, {}
			for index, answer in ipairs (answers) do
				if type (answer) == "table" and answer.kind == VIEW_WEAPON then
					view.answers_seen = view.answers_seen + 1
					replaced [index] = viewed_answer (answer)
					changed = true
				else
					replaced [index] = answer
				end
			end
			if changed then
				local copy = {}
				for key, value in pairs (request) do copy [key] = value end
				copy.answers = replaced
				view.applied = view.applied + 1
				request = copy
			end
		end
	end
	return base_aim (request)
end


-- ---------------------------------------------------------------------------------------------------------------- the run
local map = "..\\common\\maps\\map5"
local function main ()
	-- every fallible setup step is inside the owned close scope: a missing or raising factory is a failure of the witness with the evidence closed
	reference = host.model
	if type (reference) ~= "function" then hard ("this host has no host.model") end
	local terrain_factory = host.terrain_provider
	if type (terrain_factory) ~= "function" then hard ("this host has no host.terrain_provider") end
	local terrain = terrain_factory (root, map)
	base_aim = host.aim
	if type (base_aim) ~= "function" then hard ("this host has no host.aim") end
	local descriptors, order = {}, {}
	local function bind_mobile (descriptor)
		local id = host.bind_mobile (descriptor)
		descriptors [id] = { kind = descriptor.movement_kind, side = descriptor.side }
		order [#order + 1] = id
		return id
	end

	local dc = require ("eech_dc")
	dc.prepare_installation (root)
	host.use_reference_arithmetic () -- the reference host's arithmetic profile, selected right after the installation is prepared and before the boot (src/reference_arithmetic.rs)
	local engine = dc.boot {
		install_root = root, map = map, terrain = terrain, environment = host.env_provider (), campaign_directory = "camp01", campaign = "LEBANON.CHC", gunship = "apache",
		seed = seed, bind_mobile = bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
		supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = model, sense = host.sense, aim = aim,
	}
	emit { event = "boot", mode = mode, profile = profile, warmup = warmup, settle = settle, frame_ms = frame_ms, seed = seed, frame_budget = frame_budget }

	-- the original profile needs the fixture API of collision-fixture; a module without it is not the original profile
	local fixture_api = false
	do
		local ok, value = pcall (function () return engine.collision_probe end)
		fixture_api = ok and type (value) == "function"
	end
	if profile == "original" and not fixture_api then hard ("UNREACHED: profile=original, but the module has no collision-fixture API (it was not built with collision-fixture): this is not the original profile") end
	if profile == "hosted" and fixture_api then hard ("profile=hosted, but the module has the collision-fixture API: it keeps the original narrow phase and is not the shipped module (use profile=original)") end

	local clock = engine:clock ()
	local frames = 0
	-- one frame, with finite progress: the simulated clock must be a finite number that has moved forward, and the run may not use more frames than its budget
	local function frame ()
		if frames >= frame_budget then hard ("FRAME BUDGET: " .. frames .. " frames were run and the simulated clock is at " .. tostring (clock.elapsed_seconds) .. " s") end
		local was = clock.elapsed_seconds
		engine:frame (frame_ms)
		frames = frames + 1
		clock = engine:clock ()
		local now = clock.elapsed_seconds
		if not finite (now) or not (now > was) then hard ("STALLED CLOCK: a frame left the simulated clock at " .. tostring (now) .. " s from " .. tostring (was) .. " s") end
	end
	local function step (seconds_more)
		local until_time = clock.elapsed_seconds + seconds_more
		while clock.elapsed_seconds < until_time do frame () end
	end
	-- the engine's own stop, or nil. Only the anchored engine fatal is the engine's; a witness failure is re-raised as it is; anything else is a failure of the witness, not proof of anything
	local function guarded (call, ...)
		local ok, err = pcall (call, ...)
		if ok then return nil end
		local text = tostring (err)
		if string.sub (first_line (text), 1, #NAME + 1) == NAME .. ":" then error (text, 0) end
		local marker = engine_marker (text)
		if marker == nil then hard ("an unrecognised failure, not the engine's known fatal at the model boundary: " .. first_line (text)) end
		return { marker = marker, line = first_line (text) }
	end
	local function engine_fatal_text (err)
		local text = tostring (err)
		return text:find ("EECH fatal error", 1, true) ~= nil or text:find ("poisoned", 1, true) ~= nil
	end

	local setup = guarded (step, warmup)
	if setup then hard ("the engine stopped during the warmup, before anything was altered: [model-provider:" .. setup.marker .. "]") end
	emit { event = "warmed_up", elapsed = clock.elapsed_seconds, frames = frames, bound = #order, entity_before = before.entity }

	-- ------------------------------------------------------------------------------------------------------------ the case launch (preconditions, deterministic)
	local function fields (line)
		local t = {}
		for k, v in string.gmatch (line, "(%S+)=(%S+)") do t [k] = v end
		return t
	end
	local function numbers (list)
		local t = {}
		for v in string.gmatch (list, "[^,]+") do t [#t + 1] = tonumber (v) end
		return t
	end
	-- a query that fails for its arguments is a candidate that does not qualify; a failure of the ENGINE is never swallowed
	local function state (id, weapon)
		local ok, text = pcall (engine.wpn_case_query, engine, id, weapon or 0)
		if not ok then
			if engine_fatal_text (text) then hard ("the engine had stopped before the launch: " .. first_line (text)) end
			return nil
		end
		local s = fields (text)
		s.alive, s.weapon_count = tonumber (s.alive), tonumber (s.weapon_count)
		s.warhead, s.guidance, s.aiming, s.decoy = tonumber (s.warhead), tonumber (s.guidance), tonumber (s.aiming), tonumber (s.decoy)
		s.position = numbers (s.position or "")
		return s
	end
	local function events ()
		local list = {}
		for line in string.gmatch (engine:wpn_events (), "[^\n]+") do
			local f = fields (line)
			f.raw = line
			list [#list + 1] = f
		end
		return list
	end
	-- one named counter of the weapon fixture's run-wide report
	local function counter (name)
		local text = engine:wpn_trace ()
		local value = string.match (text, "wpn%-counter name=" .. name .. " value=(%d+)")
		if value == nil then hard ("the weapon fixture's report has no counter " .. name) end
		return tonumber (value)
	end

	local ordered = {}
	for _, id in ipairs (order) do ordered [#ordered + 1] = id end
	table.sort (ordered, function (a, b) return tonumber (string.match (a, "(%d+)$")) < tonumber (string.match (b, "(%d+)$")) end)
	local alive = {}
	for _, id in ipairs (ordered) do
		local s = state (id, 0)
		if s and s.alive == 1 then s.side = descriptors [id].side alive [id] = s end
	end
	-- the case: the pair of a launcher and an enemy that is a collision candidate TYPE (not a person: a person has neither collision flag) with the smallest distance, whose launcher has an
	-- unguided ballistic weapon in stock. The weapon's lookup excludes only the launcher, so the target is the weapon's first broad-phase candidate when it is within the candidate's box
	-- of radius + 10 m of the first segment; a war in which no pair is that close simply makes no narrow-phase question and the run is UNREACHED, never a pass.
	local PERSON = 3 -- host.bind_mobile movement_kind of a person
	local pair_list = {}
	for index, id in ipairs (ordered) do
		local s = alive [id]
		if s then
			local best, best_distance
			for _, other in ipairs (ordered) do
				local o = alive [other]
				if o and other ~= id and o.side ~= s.side and (o.side == "blue" or o.side == "red") and descriptors [other].kind ~= PERSON then
					local dx, dy, dz = o.position [1] - s.position [1], o.position [2] - s.position [2], o.position [3] - s.position [3]
					local distance = math.sqrt (dx * dx + dy * dy + dz * dz)
					if not best_distance or distance < best_distance then best, best_distance = other, distance end
				end
			end
			if best then pair_list [#pair_list + 1] = { launcher = id, target = best, distance = best_distance, order = index } end
		end
	end
	table.sort (pair_list, function (a, b)
		if a.distance ~= b.distance then return a.distance < b.distance end
		return a.order < b.order
	end)
	local launcher, target, weapon, distance
	for _, pair in ipairs (pair_list) do
		for w = 1, 127 do
			local q = state (pair.launcher, w)
			if q and q.weapon_count and q.weapon_count > 0 and q.guidance == 0 and q.aiming ~= 0 and q.decoy == 0 then
				launcher, target, weapon, distance = pair.launcher, pair.target, w, pair.distance
				break
			end
		end
		if launcher then break end
	end
	emit { event = "case_selected", launcher = launcher or "none", target = target or "none", weapon = weapon or -1, distance = distance or -1, pairs_considered = #pair_list,
		selection = "nearest_non_person_enemy_of_a_launcher_with_an_unguided_ballistic_weapon" }
	if not launcher then hard ("SETUP FAILURE: no launcher with an unguided ballistic weapon and a non-person enemy was found") end

	events () -- everything the natural war logged before the launch is taken first, so what is read after is the launch's own
	local any_tests_before = counter ("contact_any_tests")
	local any_hits_before = counter ("contact_any_hits")
	local precision_before = counter ("precision_high")
	armed = true
	local launched, launch_text = pcall (engine.wpn_case_launch, engine, launcher, target, weapon, 0)
	local failure
	if not launched then
		if string.sub (first_line (launch_text), 1, #NAME + 1) == NAME .. ":" then error (tostring (launch_text), 0) end
		local marker = engine_marker (launch_text)
		if marker == nil then hard ("the launch call failed, but not with the engine's known fatal at the model boundary: " .. first_line (launch_text)) end
		failure = { marker = marker, line = first_line (launch_text) }
	end

	-- the launch report joined with the births the launch call itself logged: the cohort is frozen here and never added to
	local cohort, members, report = {}, {}, nil
	if launched then
		report = fields (launch_text)
		if string.sub (launch_text, 1, #"wpn-case-launch ") ~= "wpn-case-launch " then hard ("the launch report is not a wpn-case-launch line: " .. first_line (launch_text)) end
		for _, e in ipairs (events ()) do
			if e.kind == "birth" then
				if e.w == nil or tonumber (e.w) == nil then hard ("an event with no weapon identity: " .. e.raw) end
				if cohort [e.w] then hard ("a weapon born twice in one launch: " .. e.w) end
				cohort [e.w] = true
				members [#members + 1] = e.w
			end
		end
		table.sort (members, function (a, b) return tonumber (a) < tonumber (b) end)
		local problem = launch_report_problem (report, first_line (launch_text), weapon)
		if problem then hard (problem) end
		local births = whole (report.births)
		if #members < 1 then hard ("SETUP FAILURE: the launch logged no birth: the controlled cohort is empty") end
		if births ~= #members then hard (string.format ("the launch report says births=%d but the launch logged %d birth events", births, #members)) end
	end
	emit { event = "case_launch", ok = launched and true or false, launcher = launcher, target = target, weapon = weapon, text = first_line (launch_text), births = #members }
	emit { event = "cohort", size = #members, members = table.concat (members, ",") }

	-- ------------------------------------------------------------------------------------------------------------ run until the cohort has ended, the engine stops, or the cap
	local terminal_of, terminals, foreign, duplicates, later_births = {}, {}, {}, {}, {}
	local function take ()
		for _, e in ipairs (events ()) do
			if e.kind == "birth" then
				later_births [#later_births + 1] = e.w or "?"
			elseif e.kind == "terminal" then
				local w = e.w
				if w == nil then hard ("a terminal fact with no weapon identity: " .. e.raw) end
				if not cohort [w] then foreign [#foreign + 1] = w
				elseif terminal_of [w] then duplicates [#duplicates + 1] = w
				else terminal_of [w] = e terminals [#terminals + 1] = e end
			end
		end
	end
	local function pending_members ()
		local n = 0
		for _, w in ipairs (members) do if not terminal_of [w] then n = n + 1 end end
		return n
	end
	local deadline = clock.elapsed_seconds + settle
	while not failure and pending_members () > 0 and clock.elapsed_seconds < deadline do
		failure = guarded (step, 1)
		-- after a stop NOTHING more is asked of the engine
		if failure then break end
		take ()
	end
	local any_tests_delta, any_hits_delta, precision_delta
	if not failure then
		take ()
		any_tests_delta = counter ("contact_any_tests") - any_tests_before
		any_hits_delta = counter ("contact_any_hits") - any_hits_before
		precision_delta = counter ("precision_high") - precision_before
	end

	-- the cohort's PHYSICAL signature: what each case weapon ended as and where, never its ordinal (an ordinal is bookkeeping and differs between runs for no physical reason)
	local signature_parts = {}
	for _, w in ipairs (members) do
		local e = terminal_of [w]
		if e then
			emit { event = "terminal", w = e.w, type = e.type, code = e.code, steps = e.steps, pos = e.pos, velocity = e.velocity, lifetime = e.lifetime }
			signature_parts [#signature_parts + 1] = table.concat ({ tostring (e.type), tostring (e.code), tostring (e.steps), tostring (e.pos), tostring (e.velocity), tostring (e.lifetime) }, "|")
		end
	end
	for _, w in ipairs (foreign) do emit { event = "foreign_terminal", w = w } end
	for _, w in ipairs (duplicates) do emit { event = "duplicate_terminal", w = w } end
	for _, w in ipairs (later_births) do emit { event = "later_birth", w = w } end
	local incomplete = (not failure) and pending_members () > 0
	emit {
		event = "authority", mode = mode, profile = profile, expected_marker = plan.marker or "", completed = failure == nil and not incomplete, incomplete = incomplete,
		failure_marker = failure and failure.marker or "", failure_line = failure and failure.line or "",
		cohort_size = #members, case_weapons_ended = #terminals, case_weapons_pending = failure and -1 or pending_members (), later_births = #later_births, foreign_terminals = #foreign,
		duplicate_terminals = #duplicates, elapsed = clock.elapsed_seconds, frames = frames, frame_budget = frame_budget,
		contact_any_tests_delta = any_tests_delta or -1, contact_any_hits_delta = any_hits_delta or -1, precision_high_delta = precision_delta or -1, fixture_api = fixture_api,
		view_input = "configured_view_weapon_true_on_armed_weapon_steps", view_steps_seen = view.steps_seen, view_answers_seen = view.answers_seen, view_applied = view.applied,
		view_already_set = view.already_set,
		entity_before = before.entity, entity_after = after.entity, other_ops_before = before.other, other_ops_after = after.other,
		stationary = seen.stationary, moving = seen.moving, real_hit = seen.real_hit, real_miss = seen.real_miss,
		real_mask_63 = seen.real_mask_63, real_mask_56 = seen.real_mask_56, real_mask_other = seen.real_mask_other,
		answered_hit = seen.answered_hit, answered_miss = seen.answered_miss, altered = altered,
		request_faults = faults.request, reply_faults = faults.reply, first_fault = first_fault or "",
		cohort_signature = table.concat (signature_parts, ";"),
		attribution = "global_to_the_war_no_per_weapon_identity",
		miss_coverage = (mode == "forced_miss") and "valid_accepted_miss_not_a_measured_physical_change_unless_the_control_had_hits" or "",
	}

	-- ------------------------------------------------------------------------------------------------------------ the verdict (raised after the evidence is closed by the caller)
	if plan.marker then
		-- a refusal mode: the engine must have stopped, at this marker, at an armed entity question the provider altered
		if failure == nil then return "mode " .. mode .. " completed: the engine did not stop at " .. plan.marker end
		if failure.marker ~= plan.marker then return "mode " .. mode .. " stopped at [model-provider:" .. failure.marker .. "], not at [model-provider:" .. plan.marker .. "]: " .. failure.line end
		if altered < 1 then return "mode " .. mode .. " stopped without the provider having altered an entity answer: " .. failure.line end
		if view.applied < 1 then return "mode " .. mode .. " stopped, but the controlled view input never reached a weapon step: the stop is not at an entity question of the configured flight" end
		return nil
	end
	if failure ~= nil then return "mode " .. mode .. " failed in the engine at [model-provider:" .. failure.marker .. "]: " .. failure.line end
	if incomplete then
		return string.format ("INCOMPLETE: %d of %d case weapons had no terminal by the settle cap of %g s; an unfinished cohort is never a pass", pending_members (), #members, settle)
	end
	if #duplicates > 0 then return "a case weapon ended twice (" .. #duplicates .. " duplicate terminals)" end
	for _, w in ipairs (members) do
		local e = terminal_of [w]
		local problem = terminal_problem (e)
		if problem then return "the terminal of case weapon " .. w .. " is incomplete (field " .. problem .. " is missing or not in its grammar): " .. e.raw end
	end
	-- the controlled input must have reached the host's weapon step and the step must have taken the high-precision path it opens; otherwise the counters below say nothing
	if view.applied < 1 then return string.format ("UNREACHED: the controlled view input never reached a weapon step (%d weapon steps seen, %d VIEW_WEAPON answers): the flight was not under the configured precondition", view.steps_seen, view.answers_seen) end
	if precision_delta < 1 then return "UNREACHED: the controlled view input was applied but the weapon step never took the high-precision collision path (precision_high did not rise)" end
	-- the ACTUAL caller was reached: the weapon step's entity lookup was made (the fixture's own counter, not the provider's)
	if any_tests_delta < 1 then return "UNREACHED: the weapon step made no entity-collision test (contact_any_tests did not rise): the actual entity caller was not reached" end
	if profile == "original" then
		if after.entity ~= 0 then return "profile=original, but the host was asked " .. after.entity .. " entity_collision questions: the module is not the original" end
		return "CAPABILITY RED (expected for the original profile): the entity-collision caller ran (contact_any_tests +" .. any_tests_delta .. ", contact_any_hits +" .. any_hits_delta .. ", fixture API present) and the host was asked no entity_collision question"
	end
	-- the route is the host's: a candidate that passed the (campaign) broad phase asks the host. None at all is a setup that never reached the narrow phase
	if after.entity < 1 then return string.format ("UNREACHED: the weapon step made %d entity-collision tests but no candidate reached the narrow phase, so the host was asked no entity_collision question", any_tests_delta) end
	if faults.request + faults.reply ~= 0 then return string.format ("the entity question or the host's reply is not the contract's (request_faults=%d reply_faults=%d): %s", faults.request, faults.reply, tostring (first_fault)) end
	if seen.stationary + seen.moving ~= after.entity then return string.format ("the provider counted %d entity questions but classified %d", after.entity, seen.stationary + seen.moving) end
	if mode == "control" then
		if altered ~= 0 then return "the control altered an entity answer" end
		-- every weapon-step entity hit is one narrow-phase hit of the host (the hosted line of sight asks too, so the host may report more)
		if seen.real_hit < any_hits_delta then return string.format ("the weapon step counted %d entity hits but the host's narrow phase reported only %d", any_hits_delta, seen.real_hit) end
		return nil
	end
	if altered < 1 then return "mode " .. mode .. " altered no entity answer" end
	if mode == "forced_hit" then
		if seen.answered_hit ~= after.entity or seen.answered_miss ~= 0 then return string.format ("forced_hit answered %d hits and %d misses of %d questions", seen.answered_hit, seen.answered_miss, after.entity) end
		if any_hits_delta < 1 then return "forced_hit: the forced hit never reached the weapon step's entity hit count (contact_any_hits did not rise): the answer was not used by the actual caller" end
	elseif mode == "forced_miss" then
		if seen.answered_miss ~= after.entity or seen.answered_hit ~= 0 then return string.format ("forced_miss answered %d misses and %d hits of %d questions", seen.answered_miss, seen.answered_hit, after.entity) end
		if any_hits_delta ~= 0 then return string.format ("forced_miss: the weapon step still counted %d entity hits, but the host answered every question as a miss", any_hits_delta) end
	end
	return nil
end

-- main runs under xpcall so that the evidence is closed on EVERY path; the first error is the one raised, and a close that fails is never hidden and never follows a success
local ran, verdict = xpcall (main, function (e) return e end)
local closed, close_error = out:close ()
if not ran then
	if not closed then io.stderr:write (NAME .. ": OUTPUT FAILURE while closing after the failure below: " .. tostring (close_error) .. "\n") end
	error (verdict, 0)
end
if not closed then error (NAME .. ": OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error), 0) end
if verdict ~= nil then error (NAME .. ": " .. verdict, 0) end
io.stderr:write (string.format ("%s: mode=%s profile=%s completed entity=%d other_ops=%d altered=%d\n", NAME, mode, profile, after.entity, after.other, altered))
