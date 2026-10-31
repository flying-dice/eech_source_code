-- TEST-ONLY PRIMITIVE CONTROL of the ORIGINAL lightning strike queue (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. Needs a DLL built with the `env-observer` feature (which holds the read-only
-- accessor to the queue of modules/3d/3dstrike.c); an older or smaller build refuses.
--
-- The natural run (lua/env-characterization.lua with queue=on) shows what the ORIGINAL lightning routine found and did at each add_3d_lightning_strike in the frames. It cannot show what the two primitives do
-- in the situations the frames never reach: an add with no active environment, an add to a full queue, a strike aged by the time of day. This driver runs ONE of those four controls of the fixture's group 13
-- (csrc/eech_env_fixture.c env_lightning_case) in a FRESH process, because the queue cannot be put back (the process is the isolation boundary; a control refuses a queue that has been written):
--
--   1 add_without_active_environment        the active environment is set to NULL by plain assignment, add_3d_lightning_strike is called
--   2 add_with_the_main_environment         the active environment is set to the main one by plain assignment, add_3d_lightning_strike is called
--   3 fill_the_queue_and_drop_the_next      one add more than the queue has slots for
--   4 age_a_cloud_burst_by_the_time_of_day  a CLOUD_BURST is added at a time of day and aged by update_3d_lightning_strikes at two later times
--   5 age_a_forked_strike_by_the_time_of_day  the same with a FORKED_1 strike: aging it needs the strike animation frame count, which only the texture animation table holds; where it is not loaded the ENGINE
--                                             STOPS with the named refusal [env-provider:lightning_update.model] (an engine_failure row and this driver's error, never a result); a full retail boot loads it
--                                             (the actual candidate-only run completed), and then the rows are those of case 4
--
-- Cases 1..4 characterize the two original primitives and can compare the candidate; case 5 was added after the move and is candidate-only, with no original forked conformance claimed. These are not
-- controls of the lightning routine or its switch to the main environment, and not a missing-host proof. The plan's unsupported forked note describes the original four-case scope, not case 5's outcome.
-- Forked aging needs loaded texture metadata; a full retail boot supplies it. A prerequisite that is absent (no main environment, a queue that is not empty) is REFUSED by
-- the fixture and reported here as a refusal, never as a pass and never worked around.
--
-- THIS DRIVER JUDGES NOTHING PHYSICAL. It checks the SHAPE of the fixture's report (the rows in order, the keys in order, every value of its kind, the identity of the case, the end row, an occupancy that agrees
-- with its mask) and writes every row as it came, with every value kept as text (a hex word is never turned into a number). What the original did is what the rows say. A report of another shape is a failure of
-- the WITNESS: a `witness_failure` row and an error beginning `env-lightning-queue:`. If the engine stops inside a control, that is `engine_failure` and the same error.
--
-- host.args: root, out (JSON Lines path), case (1 to 5, required), scenario (lebanon_retail default | georgia_retail)

io.stdout:setvbuf ("no")

local args = host.args
local function refuse (message) error ("env-lightning-queue: " .. message, 0) end
local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local LABELS = { "add_without_active_environment", "add_with_the_main_environment", "fill_the_queue_and_drop_the_next", "age_a_cloud_burst_by_the_time_of_day", "age_a_forked_strike_by_the_time_of_day" }
local case = tonumber (args.case)
if not case or case ~= math.floor (case) or LABELS [case] == nil then refuse ("case must be an integer from 1 to 5, got '" .. tostring (args.case) .. "'") end
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario_name = args.scenario == nil and "lebanon_retail" or args.scenario
local scenario = scenarios [scenario_name]
if scenario == nil then refuse ("scenario must be lebanon_retail or georgia_retail, got '" .. tostring (scenario_name) .. "'") end
local seed = 1
local out = assert (io.open (out_path, "w"))

local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
local function json_string (s)
	return '"' .. (string.gsub (s, '[%c"\\\128-\255]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)) .. '"'
end
-- every write and flush of the evidence is checked: a failed one is a failure of the witness
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
	if not ok then refuse ("OUTPUT FAILURE: writing " .. out_path .. " failed: " .. tostring (err)) end
	ok, err = out:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. out_path .. " failed: " .. tostring (err)) end
end
local function hard (message)
	pcall (emit, { event = "witness_failure", reason = message })
	refuse (message)
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign,
	gunship = "apache", seed = seed, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, seed = seed, case = case, label = LABELS [case], feature = "env-observer", group = 13 }

local present, method = pcall (function () return engine.env_case end)
if not present or type (method) ~= "function" then
	emit { event = "refusal", reason = "env_case_unavailable", detail = "the module was built without the env-fixture feature" }
	refuse ("the engine has no env_case method: build eech-dc with the env-observer feature")
end

-- ---------------------------------------------------------------------------------------------------------------- the report grammar: key=value rows, kept as text
local function parse (text, what)
	if type (text) ~= "string" or text == "" or string.sub (text, -1) ~= "\n" then hard (what .. " is empty or truncated (it does not end with a newline)") end
	if #text >= 32 * 1024 then hard (what .. " does not fit the fixture's 32 KiB buffer") end
	local rows = {}
	for line in string.gmatch (text, "([^\n]*)\n") do
		if line == "" or string.find (line, "[^\32-\126]") then hard (what .. " has an empty line or a byte that is not printable ASCII") end
		local event, rest = string.match (line, "^([%a][%w%-_]*)(.*)$")
		if event == nil or (rest ~= "" and string.sub (rest, 1, 1) ~= " ") then hard (what .. " has a row without an event name: " .. string.sub (line, 1, 60)) end
		local row = { event = event, raw = line, keys = {}, map = {} }
		for token in string.gmatch (rest, "%S+") do
			local k, v = string.match (token, "^([%a_][%w_]*)=([^=]+)$")
			if k == nil or row.map [k] ~= nil then hard (what .. " has a malformed or repeated field '" .. token .. "' in " .. event) end
			row.keys [#row.keys + 1] = k
			row.map [k] = v
		end
		rows [#rows + 1] = row
	end
	return rows
end
local function is_word (v) return type (v) == "string" and #v == 8 and string.find (v, "^[0-9a-f]+$") ~= nil end
local function is_count (v) return type (v) == "string" and #v <= 10 and string.find (v, "^%d+$") ~= nil end
local function is_flag (v) return v == "0" or v == "1" end
local function is_int (v) return type (v) == "string" and #v <= 11 and string.find (v, "^-?%d+$") ~= nil end
local function is_token (v) return type (v) == "string" and #v <= 200 and string.find (v, "^[%w_,]+$") ~= nil end
local KINDS = { w = is_word, n = is_count, b = is_flag, i = is_int, s = is_token }
-- "key:kind" a typed value, "key=literal" one literal; the keys in the order the C prints them
local function check (row, spec, what)
	local expected = {}
	for token in string.gmatch (spec, "%S+") do expected [#expected + 1] = token end
	if #row.keys ~= #expected then hard (what .. " row " .. row.event .. " has " .. #row.keys .. " fields, the protocol has " .. #expected .. ": " .. row.raw) end
	for n, token in ipairs (expected) do
		local k, kind = string.match (token, "^([%w_]+):(%a)$")
		if k ~= nil then
			if row.keys [n] ~= k or not KINDS [kind] (row.map [k]) then hard (what .. " row " .. row.event .. " field " .. n .. " is not '" .. k .. "' as a " .. kind .. ": " .. row.raw) end
		else
			local key, literal = string.match (token, "^([%w_]+)=(.+)$")
			if row.keys [n] ~= key or row.map [key] ~= literal then hard (what .. " row " .. row.event .. " field " .. n .. " is not " .. token .. ": " .. row.raw) end
		end
	end
end
local function popcount (word)
	local count = 0
	for i = 0, 31 do count = count + math.floor (tonumber (word, 16) / 2 ^ i) % 2 end
	return count
end
local WORDS8 = "w0:w w1:w w2:w w3:w w4:w w5:w w6:w w7:w"
local function ask (index)
	local ok, text = pcall (engine.env_case, engine, 13, index)
	if not ok then
		emit { event = "engine_failure", i = index, reason = tostring (text) }
		refuse ("env_case(13, " .. index .. ") FAILED in the engine: " .. (string.match (tostring (text), "[^\r\n]+") or tostring (text)))
	end
	if string.find (text, "env-case-error", 1, true) then hard ("env_case(13, " .. index .. ") refused: " .. text) end
	return parse (text, "env_case(13, " .. index .. ")")
end
local function write (scope, rows)
	for _, row in ipairs (rows) do
		local t = { event = row.event, scope = scope, raw = row.raw }
		for _, k in ipairs (row.keys) do t [k] = row.map [k] end
		emit (t)
	end
end

-- ---------------------------------------------------------------------------------------------------------------- the plan, then the one control
local plan = ask (0)
write ("plan", plan)
-- THE INDEX-0 REPORT, exactly as the C prints it: the plan, two unsupported rows, ONE probe report (stored nothing: this process has run no frame), then one row for every slot that is occupied NOW, in ascending
-- order. A queue that is not empty is a legitimate state: the fixture then refuses every control, explicitly, and that refusal is what is reported below.
local first = plan [1]
if first == nil or first.event ~= "env-lightning-plan" then hard ("the plan does not begin with an env-lightning-plan row") end
check (first, "version=1 cases:n main_present:b active:n queue_capacity:i mask_w:w update_calls:n process_isolated=required", "the plan")
if first.map.cases ~= tostring (#LABELS) then hard ("the plan has " .. first.map.cases .. " cases, not " .. #LABELS) end
local capacity = tonumber (first.map.queue_capacity)
if capacity < 1 or capacity > 32 then hard ("the queue accessor does not answer: queue_capacity " .. first.map.queue_capacity) end
if tonumber (first.map.active) > 2 then hard ("the plan has the active environment category " .. first.map.active .. ", which is not none, the main one or another") end
local plan_mask = tonumber (first.map.mask_w, 16)
-- the occupied slots of a mask: strictly ascending (so unique), in range, in use, as many rows as the mask has bits and no row for a bit it does not have
local function slot_cohort (slot_rows, mask_word, occupancy, what)
	local mask = tonumber (mask_word, 16)
	if popcount (mask_word) ~= tonumber (occupancy) then hard (what .. " has occupancy " .. occupancy .. " for the mask " .. mask_word) end
	local previous = -1
	for _, row in ipairs (slot_rows) do
		local slot = tonumber (row.map.slot)
		if slot >= capacity or slot <= previous then hard (what .. " has the slot row " .. row.map.slot .. " out of range, repeated or out of order") end
		if tonumber (row.map.w0, 16) == 0 or math.floor (mask / 2 ^ slot) % 2 ~= 1 then hard (what .. " has the slot row " .. row.map.slot .. " that is not an occupied slot of the mask " .. mask_word) end
		previous = slot
	end
	if #slot_rows ~= tonumber (occupancy) then hard (what .. " has " .. #slot_rows .. " slot rows for the occupancy " .. occupancy .. " of the mask " .. mask_word) end
end
local unsupported, reports, plan_slots = 0, 0, {}
local stage = 2
for n = 2, #plan do
	local row = plan [n]
	if row.event == "env-lightning-unsupported" and stage == 2 then
		check (row, "item:s reason:s", "the plan")
		unsupported = unsupported + 1
	elseif row.event == "env-lq-report" and stage <= 3 and reports == 0 then
		check (row, "armed:b capacity:n stored:n dropped:n overflow:b lightning_events:n update_calls:n queue_capacity:i final_mask_w:w final_occupancy:n", "the plan")
		reports = reports + 1
		stage = 4
		if unsupported ~= 2 then hard ("the plan has " .. unsupported .. " unsupported rows before its report, not 2") end
		local m = row.map
		if m.stored ~= "0" or m.lightning_events ~= "0" or m.dropped ~= "0" or m.final_occupancy ~= tostring (popcount (first.map.mask_w)) then hard ("the plan's report holds probe records in a process that has run no frame: " .. row.raw) end
		if m.queue_capacity ~= first.map.queue_capacity or m.final_mask_w ~= first.map.mask_w or m.update_calls ~= first.map.update_calls then hard ("the plan and its report describe different queues: " .. row.raw) end
	elseif row.event == "env-lq-slot" and stage == 4 then
		check (row, "slot:n " .. WORDS8, "the plan")
		plan_slots [#plan_slots + 1] = row
	else
		hard ("the plan has an unexpected row, or a misplaced one: " .. row.raw)
	end
end
if unsupported ~= 2 or reports ~= 1 then hard ("the plan has " .. unsupported .. " unsupported rows and " .. reports .. " reports, not 2 and 1") end
slot_cohort (plan_slots, first.map.mask_w, popcount (first.map.mask_w), "the plan")

local rows = ask (case)
write ("case", rows)
local head, last = rows [1], rows [#rows]
if head == nil or head.event ~= "env-lightning" or last == nil or last.event ~= "env-lightning-end" or #rows < 2 then hard ("the control does not run from an env-lightning row to an env-lightning-end row") end
check (head, "kind=control i:n label:s main_present:b active_before:n queue_capacity:i mask_before_w:w occupancy_before:n update_calls_before:n", "the control")
if head.map.i ~= tostring (case) or head.map.label ~= LABELS [case] then hard ("the control is not case " .. case .. " '" .. LABELS [case] .. "': " .. head.raw) end
-- the head describes the queue the plan described, in the same process
if head.map.mask_before_w ~= first.map.mask_w or head.map.main_present ~= first.map.main_present or head.map.active_before ~= first.map.active or head.map.queue_capacity ~= first.map.queue_capacity
	or head.map.update_calls_before ~= first.map.update_calls then
	hard ("the control's head is not the queue the plan described: " .. head.raw)
end
if popcount (head.map.mask_before_w) ~= tonumber (head.map.occupancy_before) then hard ("the control's occupancy does not agree with its mask: " .. head.raw) end
local refused = last.map.refused ~= nil
if refused then
	-- a prerequisite the fixture would not force: the control did not run, nothing changed, and that is what is reported. The fixture tests the queue, then the main environment (case 2 to 4 only), in that order
	check (last, "i:n refused:s changed=0 process_isolated=required", "the control")
	if #rows ~= 2 then hard ("a refused control has rows besides its head and its end: " .. #rows) end
	local reason, mask_empty = last.map.refused, head.map.mask_before_w == "00000000"
	if reason == "the_queue_is_not_empty_a_control_needs_a_process_that_has_not_queued_a_strike" then
		if mask_empty then hard ("the control refuses a queue that is not empty but its mask is empty: " .. head.raw) end
	elseif reason == "there_is_no_main_3d_environment_nothing_is_forced" then
		if not mask_empty or head.map.main_present ~= "0" or case < 2 then hard ("the control refuses for want of a main environment that exists, or a case that needs none, or after a non-empty queue: " .. head.raw) end
	else
		hard ("the control is refused for a reason the fixture does not give: " .. reason)
	end
else
	check (last, "i:n changed=1 mask_after_w:w occupancy_after:n update_calls_after:n active_restored:b queue_restored=0 process_isolated=required", "the control")
	if popcount (last.map.mask_after_w) ~= tonumber (last.map.occupancy_after) then hard ("the control's final occupancy does not agree with its mask: " .. last.raw) end
	if last.map.active_restored ~= "1" then hard ("FIXTURE DEFECT: the active environment was not put back after the control: " .. last.raw) end
	-- a control that ran started from an empty queue, and (cases 2 to 4) a main environment
	if head.map.mask_before_w ~= "00000000" or (case >= 2 and head.map.main_present ~= "1") then hard ("a control ran although its prerequisites were not met: " .. head.raw) end
	-- the rows between the head and the end, in the order the C prints them: the case's own rows, then ONE `after` row for every slot that is occupied at the end
	local at = 2
	local function expect (event, spec, what)
		local row = rows [at]
		if row == nil or row.event ~= event then hard ("case " .. case .. " has an unexpected row, or lacks its " .. what .. " row at position " .. at) end
		check (row, spec, "the control")
		at = at + 1
		return row
	end
	local updates = 0
	if case == 2 then
		expect ("env-lightning-input", "tag=add main_time_w:w type:n length_w:w x_w:w z_w:w", "input")
	elseif case == 3 then
		-- the typed step rows, their case and add ordinals, each occupancy its mask's, and the last mask the control's final mask. What the masks show (a slot gained, kept or lost between the two adds) is the
		-- original's behaviour and is kept as it came: no outcome is preferred here and none is rejected
		for k = 0, 1 do
			local step = expect ("env-lightning-step", "i:n add:n mask_w:w occupancy:n", "step")
			if step.map.i ~= tostring (case) or tonumber (step.map.add) ~= capacity + k then hard ("step row " .. (k + 1) .. " is not add " .. (capacity + k) .. " of case " .. case .. ": " .. step.raw) end
			if popcount (step.map.mask_w) ~= tonumber (step.map.occupancy) then hard ("step row " .. (k + 1) .. " has an occupancy that is not its mask's: " .. step.raw) end
			if k == 1 and step.map.mask_w ~= last.map.mask_after_w then hard ("the last step's mask is not the control's final mask: " .. step.raw) end
		end
	elseif case == 4 or case == 5 then
		local input = expect ("env-lightning-input", "tag=age added_at_w:w length_w:w inside_w:w after_the_end_w:w first_slot:i", "input")
		updates = 2
		local first_slot = tonumber (input.map.first_slot)
		if first_slot >= capacity or first_slot < -1 then hard ("the aging input names slot " .. input.map.first_slot .. " of a queue of " .. capacity) end
		if first_slot >= 0 then
			-- the three snapshots of the slot the add occupied, in order
			for n, tag in ipairs ({ "added", "aged_inside_the_length", "aged_after_the_end" }) do
				local row = expect ("env-lightning-slot", "tag:s slot:n " .. WORDS8, "tagged slot")
				if row.map.tag ~= tag or tonumber (row.map.slot) ~= first_slot then hard ("tagged slot row " .. n .. " is not '" .. tag .. "' of slot " .. first_slot .. ": " .. row.raw) end
				if tag == "added" and tonumber (row.map.w0, 16) == 0 then hard ("the slot the add occupied is not in use in its 'added' row: " .. row.raw) end
			end
		end
	end
	local after_rows = {}
	while at < #rows do
		local row = expect ("env-lightning-slot", "tag:s slot:n " .. WORDS8, "final slot")
		if row.map.tag ~= "after" then hard ("a final slot row is tagged '" .. row.map.tag .. "', not 'after': " .. row.raw) end
		after_rows [#after_rows + 1] = row
	end
	slot_cohort (after_rows, last.map.mask_after_w, last.map.occupancy_after, "the control's final queue")
	if tonumber (last.map.update_calls_after) ~= tonumber (head.map.update_calls_before) + updates then
		hard ("the control ran " .. (tonumber (last.map.update_calls_after) - tonumber (head.map.update_calls_before)) .. " updates of the strike queue, not the " .. updates .. " its case calls")
	end
end
if last.map.i ~= tostring (case) then hard ("the end row is of another case: " .. last.raw) end
emit { event = "result", case = case, label = LABELS [case], refused = refused, refusal = last.map.refused or "", changed = last.map.changed or "0", mask_before_w = head.map.mask_before_w,
	mask_after_w = last.map.mask_after_w or head.map.mask_before_w, occupancy_before = head.map.occupancy_before, occupancy_after = last.map.occupancy_after or head.map.occupancy_before,
	update_calls_before = head.map.update_calls_before, update_calls_after = last.map.update_calls_after or head.map.update_calls_before }

-- the last thing the witness does is close its evidence, and a close that fails is a failure after the last row as much as before it
local closed, close_error = out:close ()
if not closed then refuse ("OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error)) end
io.stderr:write (string.format ("env-lightning-queue: case=%d label=%s refused=%s\n", case, LABELS [case], tostring (refused)))
