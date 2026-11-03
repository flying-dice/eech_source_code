-- TEST-ONLY host-authority and refusal witness of the production TREE-collision handoff (EEDCS-49, D3). Needs a DLL built with the `weapon-fixture` feature (the controlled launch of
-- lua/weapon-case.lua) and an eech-world with `host.terrain_provider` and `host.create_exclusive`.
--
-- Boundary under test (the product path, not a private test of Rust or C): host-owned Lua -> the public native module -> the host's terrain provider -> back into a REAL weapon launched
-- through the campaign's own launch call, whose hosted flight asks the campaign for the tree lookup after every entity test (wn_move.c CAMPAIGN_STEP_LOOKUP_TREES) and whose
-- get_line_of_sight_collision_tree answers from the HOST: tree_begin and tree_terrain are provider operations and the module has no tree data, scan or intersection behind them.
--
-- TWO PROFILES, named by `profile=`:
--   hosted (default)   the module as shipped: built with `weapon-fixture` and NOT `collision-fixture`.
--   original           the explicitly original characterisation: built with `weapon-fixture` AND `collision-fixture`, whose tree solver is the unmoved original (EECH_CAMPAIGN_HOST_TREE is not defined).
--                      Only mode=control. It is the capability RED of the host route: the weapon step made real tree tests (the fixture's contact_trees_tests rose), the fixture API is present, and
--                      the host was asked no tree question. That is reported as `CAPABILITY RED` and the process FAILS on purpose (the expected red). A module without the fixture API, or a run
--                      in which the weapon step made no tree test, is `UNREACHED`: the call this RED is about never happened, which is setup and not a red.
--
-- One bounded fresh war per process. The provider is the host's own (host.terrain_provider), delegated to unchanged and counted, until the case weapon is launched ("armed"); from
-- then on the chosen mode may answer a tree question differently:
--
--   control           nothing is altered: the healthy same-DLL control. Self-asserted from what the provider SEES after the launch: the host was asked (tree_begin count equals the weapon
--                     step's own contact_trees_tests delta, so every tree test reached the host and none was answered elsewhere); every NEED_TERRAIN names an end the dialogue allows
--                     (none, OLD only, NEW only for a segment that enters the map, or OLD then NEW; never twice, never NEW then OLD); each is answered by exactly ONE terrain `point` read at the
--                     asked coordinates and then tree_terrain carrying THAT read's elevation pattern; a second read in one query carries the hint the first one returned (the shared record).
--   forced_hit        every armed tree_begin is answered DONE { hit = true } at the segment's OLD position with an up normal, without a terrain question: a VALID changed answer. The
--                     cohort's terminal facts must change against the control (compare the two outputs: the case weapon ends at a tree contact where the control does not).
--   new_only          every armed tree_begin is answered NEED_TERRAIN for the NEW end, then DONE (miss): a VALID sequence for a segment that enters the map. The dialogue must accept it
--                     (the campaign made exactly one terrain read per query, at the NEW end) and the run completes. The old dialogue stopped here at tree.lookups.
--   withheld_tree     the provider raises on the first armed tree_begin:                                               [terrain-provider:tree_begin.callback]
--   bad_tree_state    tree_begin answers state = 'waiting':                                                            [terrain-provider:tree_begin.answer]
--   bad_tree_word     tree_begin answers NEED_TERRAIN with an x that is a Lua number, not a pattern:                   [terrain-provider:tree_begin.answer]
--   done_not_boolean  tree_begin answers DONE with hit = 1:                                                            [terrain-provider:tree_begin.answer]
--   new_then_old      tree_begin asks NEW, and tree_terrain then asks OLD:                                              [terrain-provider:tree.lookups]
--   old_twice         tree_begin asks OLD, and tree_terrain asks OLD again:                                             [terrain-provider:tree.lookups]
--   third_lookup      tree_begin asks OLD, tree_terrain asks NEW, and the next tree_terrain asks NEW again:             [terrain-provider:tree.lookups]
--
-- WHAT COUNTS AS THE ENGINE STOPPING: only the first line of the failure, anchored: `runtime error: eech: EECH fatal error: host terrain: [terrain-provider:<marker>] `. A Lua error that merely
-- mentions a marker, a witness failure, a failure without that wrapper and a different marker (a callback cause where an answer cause was expected) are failures of the witness, never proof. After
-- the stop NOTHING is asked of the engine again (no census, report, query or restoration), and no result of the campaign continuing is claimed.
--
-- WHAT COUNTS AS A COMPLETED CASE: the launch report (`wpn-case-launch`) is parsed and joined with the births the launch call logged: a non-empty cohort, births equal to the cohort, a creation, and
-- the temporary target link restored (or never changed). The run then needs every cohort member to end with exactly one complete terminal before the settle cap. A cap reached with a member
-- pending is `INCOMPLETE` and fails; it is never a pass. Terminals of weapons outside the cohort (natural activity of the war) are declared as foreign and are not attributed to the case; the
-- provider's questions are NOT attributed to the case weapons either (the war's other weapons ask too), which is why the control relates them to the weapon step's own test counter.
--
-- The evidence file is created exclusively (never truncated) with `host.create_exclusive`, whose handle observes the platform close (src/exclusive_output.rs; the older `host.output_create_new` only
-- syncs and leaves the close to an unchecked drop, so it is not used). Everything that can fail after the creation (the terrain factory included) runs inside the owned close scope; the evidence is
-- closed on every path and the first error is the one raised. Exit 0 is the verdict.
--
-- host.args: root, mode (default control), profile (default hosted), warmup (simulated seconds before the launch, default 30), settle (cap on the simulated seconds after it, default 120),
--            frame_ms (default 100), out (JSON Lines path, must not exist)

io.stdout:setvbuf ("no")

local NAME = "tree-authority"
local function refuse (message) error (NAME .. ": " .. message, 0) end

local huge = math.huge
local function finite (v) return type (v) == "number" and v == v and v ~= huge and v ~= -huge end

-- BEGIN words
-- ---------------------------------------------------------------------------------------------------------------- float patterns
-- NOTHING HERE DIVIDES. The native host's floating point policy after boot traps a divide by zero in hardware (the sign test `1 / x < 0` of a zero ended the rotor witness's public controls with
-- 0xC000008E, and `pcall` cannot catch a trap), so the sign of a zero is read from its printed form and the two halves of a word are taken by multiplying with an exact power of two.
-- tests/tree_authority_words.rs runs THIS text with the trap enabled in a child process.
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
local function asks (end_name) return { state = "need_terrain", ["end"] = end_name, x = ZERO, z = ZERO } end
local function miss (request) return { state = "done", hit = false, point = request.point, normal = request.normal } end
local MODES = {
	control = { handlers = {} },
	forced_hit = {
		handlers = { tree_begin = function (request) return { state = "done", hit = true, point = request.old, normal = ZERO .. ",0x3f800000," .. ZERO } end },
		altering = true,
	},
	new_only = {
		-- NEW first and then the miss, whatever the segment: a valid sequence; the coordinates are the segment's own NEW end, which the request carries
		handlers = {
			tree_begin = function (request, _, state)
				local x, z = string.match (request.new, "^(0x%x%x%x%x%x%x%x%x),0x%x%x%x%x%x%x%x%x,(0x%x%x%x%x%x%x%x%x)$")
				state.request = request
				return { state = "need_terrain", ["end"] = "new", x = x, z = z }
			end,
			tree_terrain = function (_, _, state) return miss (state.request) end,
		},
		altering = true,
	},
	withheld_tree = { handlers = { tree_begin = function () error ("the host withholds its tree answer", 0) end }, marker = "tree_begin.callback" },
	bad_tree_state = { handlers = { tree_begin = function () return { state = "waiting" } end }, marker = "tree_begin.answer" },
	bad_tree_word = { handlers = { tree_begin = function () return { state = "need_terrain", ["end"] = "old", x = 1.5, z = ZERO } end }, marker = "tree_begin.answer" },
	done_not_boolean = {
		handlers = { tree_begin = function () return { state = "done", hit = 1, point = ZERO .. "," .. ZERO .. "," .. ZERO, normal = ZERO .. "," .. ZERO .. "," .. ZERO } end },
		marker = "tree_begin.answer",
	},
	new_then_old = {
		handlers = {
			tree_begin = function () return asks ("new") end,
			tree_terrain = function () return asks ("old") end,
		},
		marker = "tree.lookups",
	},
	old_twice = {
		handlers = {
			tree_begin = function () return asks ("old") end,
			tree_terrain = function () return asks ("old") end,
		},
		marker = "tree.lookups",
	},
	third_lookup = {
		handlers = {
			tree_begin = function () return asks ("old") end,
			tree_terrain = function () return asks ("new") end,
		},
		marker = "tree.lookups",
	},
}
local plan = MODES [mode]
if plan == nil then refuse ("unknown mode '" .. tostring (mode) .. "'") end
if profile == "original" and mode ~= "control" then refuse ("profile=original runs only mode=control: no tree question reaches the provider to alter") end

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
local FATAL = "^runtime error: eech: EECH fatal error: host terrain: %[terrain%-provider:([%w_%.]+)%] "
local function engine_marker (err) return string.match (first_line (err), FATAL) end

-- ---------------------------------------------------------------------------------------------------------------- the host's provider, counted, altered only when armed
local armed = false
local before, after = { point = 0, tree_begin = 0, tree_terrain = 0 }, { point = 0, tree_begin = 0, tree_terrain = 0 }
local altered = { tree_begin = 0, tree_terrain = 0 }
local map = "..\\common\\maps\\map5"
local real -- the host's own provider, built inside the owned close scope (main)
local handler_state = {}
-- what the provider SEES after the launch: a tree query is a begin, then per NEED_TERRAIN one point read and one tree_terrain carrying that read's elevation pattern
local queries, need_terrain_replies, done_replies = 0, 0, 0
local order_faults, pairing_faults, coordinate_faults, hint_faults = 0, 0, 0, 0
local ends_seen = { none = 0, old = 0, new = 0, old_new = 0 }
local pending = nil
local function coordinate_matches (value, expected)
	local seen = word_of (value)
	if seen == nil then return false end
	if "0x" .. seen == expected then return true end
	-- a value of zero reaches the provider as a Lua integer (the bridge hands a double that mlua reads back as an integer), which carries no sign and no subnormal: any zero or subnormal
	-- word (exponent bits all zero) stands for it
	if value == 0 then
		local high = tonumber (string.sub (expected, 3, 6), 16)
		return high ~= nil and high % 32768 < 128
	end
	return false
end
local function note_reply (answer)
	if type (answer) ~= "table" then return end
	if answer.state == "need_terrain" then
		need_terrain_replies = need_terrain_replies + 1
		local e = answer ["end"]
		-- the dialogue's rule: an end at most once, never OLD after NEW
		if (e ~= "old" and e ~= "new") or pending.asked [e] or (e == "old" and pending.asked.new) then order_faults = order_faults + 1 end
		pending.waiting, pending.reads, pending.read_bits, pending.x, pending.z = e, 0, nil, answer.x, answer.z
	elseif answer.state == "done" then
		done_replies = done_replies + 1
		local old, new = pending.asked.old, pending.asked.new
		local key = (old and new) and "old_new" or old and "old" or new and "new" or "none"
		ends_seen [key] = ends_seen [key] + 1
		pending = nil
	end
end
local function provider (request)
	local op = request.op
	local counts = armed and after or before
	if counts [op] ~= nil then counts [op] = counts [op] + 1 end
	local handler = armed and plan.handlers [op]
	local answer
	if handler then
		altered [op] = altered [op] + 1
		answer = handler (request, real, handler_state)
	else
		answer = real (request)
	end
	if armed then
		if op == "tree_begin" then
			queries = queries + 1
			pending = { asked = {}, last_hint = nil, reads_total = 0 }
			note_reply (answer)
		elseif op == "tree_terrain" and pending ~= nil then
			-- the elevation the campaign carries must be the one the preceding point read answered, and exactly one read was made for this lookup
			local expected = pending.read_bits and ("0x" .. pending.read_bits)
			if pending.reads ~= 1 or expected == nil or request.elevation ~= expected then pairing_faults = pairing_faults + 1 end
			if pending.waiting then pending.asked [pending.waiting] = true end
			pending.waiting = nil
			note_reply (answer)
		elseif op == "point" and pending ~= nil and pending.waiting ~= nil and type (answer) == "table" then
			pending.reads = pending.reads + 1
			pending.read_bits = word_of (answer.elevation)
			-- the read is at the coordinates the host asked for
			if not coordinate_matches (request.x, pending.x) or not coordinate_matches (request.z, pending.z) then coordinate_faults = coordinate_faults + 1 end
			-- the second read of one query reads the record the first left: its request carries the hint the first answer returned
			if pending.reads_total >= 1 and request.hint ~= pending.last_hint then hint_faults = hint_faults + 1 end
			pending.reads_total = pending.reads_total + 1
			pending.last_hint = answer.hint
		end
	end
	return answer
end

-- ---------------------------------------------------------------------------------------------------------------- the run
local function main ()
	-- every fallible setup step is inside the owned close scope: a missing or raising factory is a failure of the witness with the evidence closed
	local factory = host.terrain_provider
	if type (factory) ~= "function" then hard ("this host has no host.terrain_provider") end
	real = factory (root, map)
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
		install_root = root, map = map, terrain = provider, environment = host.env_provider (), campaign_directory = "camp01", campaign = "LEBANON.CHC", gunship = "apache",
		seed = seed, bind_mobile = bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
		supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = host.model, sense = host.sense, aim = host.aim,
	}
	emit { event = "boot", mode = mode, profile = profile, warmup = warmup, settle = settle, frame_ms = frame_ms, seed = seed, frame_budget = frame_budget }

	-- the original profile needs the fixture API of collision-fixture; a module without it is not the original profile
	local fixture_api = false
	do
		local ok, value = pcall (function () return engine.collision_probe end)
		fixture_api = ok and type (value) == "function"
	end
	if profile == "original" and not fixture_api then hard ("UNREACHED: profile=original, but the module has no collision-fixture API (it was not built with collision-fixture): this is not the original profile") end
	if profile == "hosted" and fixture_api then hard ("profile=hosted, but the module has the collision-fixture API: it keeps the original solver and is not the shipped module (use profile=original)") end

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
		if marker == nil then hard ("an unrecognised failure, not the engine's known fatal at the terrain boundary: " .. first_line (text)) end
		return { marker = marker, line = first_line (text) }
	end
	local function engine_fatal_text (err)
		local text = tostring (err)
		return text:find ("EECH fatal error", 1, true) ~= nil or text:find ("poisoned", 1, true) ~= nil
	end

	local setup = guarded (step, warmup)
	if setup then hard ("the engine stopped during the warmup, before anything was altered: [terrain-provider:" .. setup.marker .. "]") end
	emit { event = "warmed_up", elapsed = clock.elapsed_seconds, frames = frames, bound = #order, tree_before = before.tree_begin }

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
	local function nearest_enemy (id)
		local s, best, best_d = alive [id], nil, nil
		for _, other in ipairs (ordered) do
			local o = alive [other]
			if o and other ~= id and o.side ~= s.side and (o.side == "blue" or o.side == "red") then
				local dx, dz = o.position [1] - s.position [1], o.position [3] - s.position [3]
				local d = math.sqrt (dx * dx + dz * dz)
				if not best_d or d < best_d then best, best_d = other, d end
			end
		end
		return best
	end
	-- the first launcher (in roster order) with an unguided ballistic weapon in stock and an enemy to aim at: a weapon that flies a ballistic path to the ground or its target
	local launcher, target, weapon
	for _, id in ipairs (ordered) do
		if alive [id] then
			local enemy = nearest_enemy (id)
			for w = 1, 127 do
				local q = enemy and state (id, w)
				if q and q.weapon_count and q.weapon_count > 0 and q.guidance == 0 and q.aiming ~= 0 and q.decoy == 0 then launcher, target, weapon = id, enemy, w break end
			end
		end
		if launcher then break end
	end
	emit { event = "case_selected", launcher = launcher or "none", target = target or "none", weapon = weapon or -1 }
	if not launcher then hard ("SETUP FAILURE: no launcher with an unguided ballistic weapon and an enemy was found") end

	events () -- everything the natural war logged before the launch is taken first, so what is read after is the launch's own
	local tests_before = counter ("contact_trees_tests")
	armed = true
	local launched, launch_text = pcall (engine.wpn_case_launch, engine, launcher, target, weapon, 0)
	local failure
	if not launched then
		if string.sub (first_line (launch_text), 1, #NAME + 1) == NAME .. ":" then error (tostring (launch_text), 0) end
		local marker = engine_marker (launch_text)
		if marker == nil then hard ("the launch call failed, but not with the engine's known fatal at the terrain boundary: " .. first_line (launch_text)) end
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
	local tests_delta
	if not failure then
		take ()
		tests_delta = counter ("contact_trees_tests") - tests_before
	end

	for _, w in ipairs (members) do
		local e = terminal_of [w]
		if e then emit { event = "terminal", w = e.w, type = e.type, code = e.code, steps = e.steps, pos = e.pos, velocity = e.velocity, lifetime = e.lifetime } end
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
		tests_delta = tests_delta or -1, fixture_api = fixture_api,
		tree_begin_before = before.tree_begin, tree_begin_after = after.tree_begin, tree_terrain_before = before.tree_terrain, tree_terrain_after = after.tree_terrain,
		point_after = after.point, queries = queries, need_terrain_replies = need_terrain_replies, done_replies = done_replies, order_faults = order_faults,
		pairing_faults = pairing_faults, coordinate_faults = coordinate_faults, hint_faults = hint_faults, altered_begin = altered.tree_begin, altered_terrain = altered.tree_terrain,
		ends_none = ends_seen.none, ends_old = ends_seen.old, ends_new = ends_seen.new, ends_old_new = ends_seen.old_new,
	}

	-- ------------------------------------------------------------------------------------------------------------ the verdict (raised after the evidence is closed by the caller)
	if plan.marker then
		-- a refusal mode: the engine must have stopped, at this marker, at an armed tree question the provider altered
		if failure == nil then return "mode " .. mode .. " completed: the engine did not stop at " .. plan.marker end
		if failure.marker ~= plan.marker then return "mode " .. mode .. " stopped at [terrain-provider:" .. failure.marker .. "], not at [terrain-provider:" .. plan.marker .. "]: " .. failure.line end
		if altered.tree_begin + altered.tree_terrain < 1 then return "mode " .. mode .. " stopped without the provider having altered a tree answer: " .. failure.line end
		return nil
	end
	if failure ~= nil then return "mode " .. mode .. " failed in the engine at [terrain-provider:" .. failure.marker .. "]: " .. failure.line end
	if incomplete then
		return string.format("INCOMPLETE: %d of %d case weapons had no terminal by the settle cap of %g s; an unfinished cohort is never a pass", pending_members (), #members, settle)
	end
	if #duplicates > 0 then return "a case weapon ended twice (" .. #duplicates .. " duplicate terminals)" end
	for _, w in ipairs (members) do
		local e = terminal_of [w]
		local problem = terminal_problem (e)
		if problem then return "the terminal of case weapon " .. w .. " is incomplete (field " .. problem .. " is missing or not in its grammar): " .. e.raw end
	end
	if profile == "original" then
		if after.tree_begin ~= 0 then return "profile=original, but the host was asked " .. after.tree_begin .. " tree questions: the module is not the original" end
		if tests_delta < 1 then return "UNREACHED: the weapon step made no tree test (contact_trees_tests did not rise): the original tree call this RED is about never happened" end
		return "CAPABILITY RED (expected for the original profile): the original tree solver ran (contact_trees_tests +" .. tests_delta .. ", fixture API present) and the host was asked no tree question"
	end
	-- the route is the host's: every tree test of the weapon step reached the provider
	if tests_delta < 1 then return "UNREACHED: the weapon step made no tree test, so no tree question was due" end
	if after.tree_begin ~= tests_delta then return string.format("the weapon step made %d tree tests but the host was asked %d tree questions: some were answered elsewhere", tests_delta, after.tree_begin) end
	if mode == "control" or mode == "new_only" then
		if order_faults + pairing_faults + coordinate_faults + hint_faults ~= 0 then
			return string.format ("the tree dialogue is not the original's: order_faults=%d pairing_faults=%d coordinate_faults=%d hint_faults=%d", order_faults, pairing_faults, coordinate_faults, hint_faults)
		end
	end
	if mode == "control" and altered.tree_begin + altered.tree_terrain ~= 0 then return "the control altered a tree answer" end
	if plan.altering and altered.tree_begin < 1 then return "mode " .. mode .. " altered no tree answer" end
	if mode == "new_only" and (ends_seen.new ~= queries or ends_seen.none + ends_seen.old + ends_seen.old_new ~= 0) then
		return string.format ("new_only: every query should have asked the NEW end only, got none=%d old=%d new=%d old_new=%d of %d", ends_seen.none, ends_seen.old, ends_seen.new, ends_seen.old_new, queries)
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
io.stderr:write (string.format ("%s: mode=%s profile=%s completed tree_begin=%d tree_terrain=%d altered=%d/%d\n", NAME, mode, profile, after.tree_begin, after.tree_terrain, altered.tree_begin, altered.tree_terrain))
