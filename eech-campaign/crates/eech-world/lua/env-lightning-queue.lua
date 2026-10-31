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
--
-- These are controls of the two ORIGINAL PRIMITIVES, not of the lightning routine, not a missing-host proof, and they do not exercise the routine's own switch to the main environment. The forked strike
-- types read texture animations the headless build may not initialise and are NOT exercised (the report says so). A prerequisite that is absent (no main environment, a queue that is not empty) is REFUSED by
-- the fixture and reported here as a refusal, never as a pass and never worked around.
--
-- THIS DRIVER JUDGES NOTHING PHYSICAL. It checks the SHAPE of the fixture's report (the rows in order, the keys in order, every value of its kind, the identity of the case, the end row, an occupancy that agrees
-- with its mask) and writes every row as it came, with every value kept as text (a hex word is never turned into a number). What the original did is what the rows say. A report of another shape is a failure of
-- the WITNESS: a `witness_failure` row and an error beginning `env-lightning-queue:`. If the engine stops inside a control, that is `engine_failure` and the same error.
--
-- host.args: root, out (JSON Lines path), case (1 to 4, required), scenario (lebanon_retail default | georgia_retail)

io.stdout:setvbuf ("no")

local args = host.args
local function refuse (message) error ("env-lightning-queue: " .. message, 0) end
local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local LABELS = { "add_without_active_environment", "add_with_the_main_environment", "fill_the_queue_and_drop_the_next", "age_a_cloud_burst_by_the_time_of_day" }
local case = tonumber (args.case)
if not case or case ~= math.floor (case) or LABELS [case] == nil then refuse ("case must be an integer from 1 to 4, got '" .. tostring (args.case) .. "'") end
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
local first = plan [1]
if first == nil or first.event ~= "env-lightning-plan" then hard ("the plan does not begin with an env-lightning-plan row") end
check (first, "version=1 cases:n main_present:b active:n queue_capacity:i mask_w:w update_calls:n process_isolated=required", "the plan")
if first.map.cases ~= tostring (#LABELS) then hard ("the plan has " .. first.map.cases .. " cases, not " .. #LABELS) end
if tonumber (first.map.queue_capacity) < 1 or tonumber (first.map.queue_capacity) > 32 then hard ("the queue accessor does not answer: queue_capacity " .. first.map.queue_capacity) end
local unsupported = 0
for n = 2, #plan do
	local row = plan [n]
	if row.event == "env-lightning-unsupported" then
		check (row, "item:s reason:s", "the plan")
		unsupported = unsupported + 1
	elseif row.event ~= "env-lq-report" then
		hard ("the plan has an unexpected row: " .. row.raw)
	end
end
if unsupported ~= 2 then hard ("the plan has " .. unsupported .. " unsupported rows, not 2") end

local rows = ask (case)
write ("case", rows)
local head, last = rows [1], rows [#rows]
if head == nil or head.event ~= "env-lightning" or last == nil or last.event ~= "env-lightning-end" or #rows < 2 then hard ("the control does not run from an env-lightning row to an env-lightning-end row") end
check (head, "kind=control i:n label:s main_present:b active_before:n queue_capacity:i mask_before_w:w occupancy_before:n update_calls_before:n", "the control")
if head.map.i ~= tostring (case) or head.map.label ~= LABELS [case] then hard ("the control is not case " .. case .. " '" .. LABELS [case] .. "': " .. head.raw) end
if popcount (head.map.mask_before_w) ~= tonumber (head.map.occupancy_before) then hard ("the control's occupancy does not agree with its mask: " .. head.raw) end
local refused = last.map.refused ~= nil
if refused then
	-- a prerequisite the fixture would not force: the control did not run, nothing changed, and that is what is reported
	check (last, "i:n refused:s changed=0 process_isolated=required", "the control")
	if #rows ~= 2 then hard ("a refused control has rows besides its head and its end: " .. #rows) end
else
	check (last, "i:n changed=1 mask_after_w:w occupancy_after:n update_calls_after:n active_restored:b queue_restored=0 process_isolated=required", "the control")
	if popcount (last.map.mask_after_w) ~= tonumber (last.map.occupancy_after) then hard ("the control's final occupancy does not agree with its mask: " .. last.raw) end
	if last.map.active_restored ~= "1" then hard ("FIXTURE DEFECT: the active environment was not put back after the control: " .. last.raw) end
	local slots, steps = 0, 0
	for n = 2, #rows - 1 do
		local row = rows [n]
		if row.event == "env-lightning-slot" then
			check (row, "tag:s slot:n " .. WORDS8, "the control")
			slots = slots + 1
		elseif row.event == "env-lightning-step" then
			check (row, "i:n add:n mask_w:w occupancy:n", "the control")
			steps = steps + 1
		elseif row.event == "env-lightning-input" then
			if #row.keys < 2 or row.map.tag == nil then hard ("the control's input row has no tag: " .. row.raw) end
		else
			hard ("the control has an unexpected row: " .. row.raw)
		end
	end
	if case == 3 and steps ~= 2 then hard ("case 3 has " .. steps .. " step rows, not 2") end
	if case ~= 3 and steps ~= 0 then hard ("case " .. case .. " has step rows") end
end
if last.map.i ~= tostring (case) then hard ("the end row is of another case: " .. last.raw) end
emit { event = "result", case = case, label = LABELS [case], refused = refused, refusal = last.map.refused or "", changed = last.map.changed or "0", mask_before_w = head.map.mask_before_w,
	mask_after_w = last.map.mask_after_w or head.map.mask_before_w, occupancy_before = head.map.occupancy_before, occupancy_after = last.map.occupancy_after or head.map.occupancy_before,
	update_calls_before = head.map.update_calls_before, update_calls_after = last.map.update_calls_after or head.map.update_calls_before }

-- the last thing the witness does is close its evidence, and a close that fails is a failure after the last row as much as before it
local closed, close_error = out:close ()
if not closed then refuse ("OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error)) end
io.stderr:write (string.format ("env-lightning-queue: case=%d label=%s refused=%s\n", case, LABELS [case], tostring (refused)))
