-- TEST-ONLY host-authority and refusal witness of the host-owned AMBIENT sound amplitude (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
-- Needs an eech-dc built with the `env-fixture` feature (the fixture's group 11 calls the real routine) and a campaign built with the host ambient move (ss_updt.c hosted routine).
--
-- The campaign's session sound routine (ss_updt.c update_session_sound_effects) no longer computes the rain, wind or terrain-smoothed amplitude of its ambient sound children: it asks the HOST. The
-- environment provider answers `op = "ambient"` (rain light, rain heavy, wind light, wind heavy) and the terrain provider answers `op = "ambient"` (one amplitude per terrain ambient child it was asked about, in the
-- order asked). This driver shows, through the public path (host-owned Lua -> `require ("eech_dc")` -> the real campaign/session routine -> the real sound-effect children), that the amplitudes the children
-- hold are the host's:
--
--   control                   both providers delegated unchanged to the REFERENCE host (host.env_provider, host.terrain_provider); every ambient request and the reference's reply are recorded. The children must hold
--                             exactly what the reference answered.
--   changed_env_ambient       the environment provider answers EVERY ambient question { rain_light = 0.375, rain_heavy = 0.1875, wind_light = 0.625, wind_heavy = 0.3125 }: a VALID, finite, distinct dyadic answer the
--                             reference would not give. The four wind and rain children must hold exactly those words.
--   changed_terrain_ambient   the terrain provider answers EVERY ambient question with one DISTINCT dyadic amplitude (4 i + 1) / 256 for the i-th child it was asked about. Each real terrain child, in the order asked, must
--                             hold exactly ITS amplitude: the mapping to the child classes and the order, not a request count.
--   withheld_env_ambient      the environment provider raises:                               [env-provider:ambient.callback]
--   malformed_env_ambient     a wind level of 1.5 (outside 0 to 1):                           [env-provider:ambient.answer]
--   withheld_terrain_ambient  the terrain provider raises:                                    [terrain-provider:ambient.callback]
--   malformed_terrain_ambient one amplitude fewer than the children asked about:              [terrain-provider:ambient.amplitudes]
--
-- ONLY `op = "ambient"` is altered, and only while the selected case runs (armed just before it); every other operation of both providers (the weather and wind queries the fixture echoes, the sector question,
-- the map facts) is delegated to the reference unchanged and the real host is not asked for an altered ambient question. The selected case is the fixture's group 11 case, UNCHANGED, with its per-child amplification words
-- (docs/d3-environment-sound-routine.md); the ONE selected case is 27 `terrain_sea_rising_from_zero` (one call of the routine, all ten ambient children present, outdoors). The children are read in the order the
-- REAL session list holds them -- the constructor inserts each new sound child at the HEAD of the list, so it is the reverse of the order the campaign attaches them: amb4, amb3, amb2, amb1, jungle, sea, heavy_wind,
-- light_wind, heavy_rain, light_rain (the terrain provider is asked about the six terrain children in that list order, classes 5, 4, 3, 2, 1, 0). The records of group 11 carry ordinal:subtype:class:word, never an entity id.
--
-- WHAT THIS SHOWS AND WHAT IT DOES NOT. It shows, for the ambient amplitude of that fixture case, that the answer is the host's, mapped to the right real children in the right order, and that a withheld or malformed answer stops the
-- campaign at the named new boundary with no fallback to a campaign solver. A refusal run stops the ENGINE (it is reported as such: the fixture never returns and no case row exists). It is a focused fixture control: it is not a real
-- frame or update_server run, it does not observe the sound-effect update, any speaker, network delivery, the player, the renderer, Tacview or the shipping module, and nothing is inferred about them.
--
-- Rows (JSON Lines): boot, plan, ambient (every ambient question, up to 4 per provider: what was asked and what was sent or answered), case (the fixture's per-child words), authority (the counters). A failure of the ENGINE is
-- an `authority` row with completed = false and the engine's first error line; a failure of the WITNESS (a malformed fixture text, an output failure) raises an error beginning `env-ambient-authority:` after a
-- best-effort `witness_failure` row.
--
-- The fixture's record of the selected case is validated as the mature characterization driver validates its own (plan version, camera and completeness; the case, session and call records and their 0/1 facts; the
-- ordinal:subtype:class:word tuples before and after the call; the restoration facts and the sixteen-digit rendered-state fingerprints, which are hashes of the rendered state and not a floating-point proof) and the
-- case row retains the raw record so that the judge can reconcile the summary with it. A malformed record is a failure of the WITNESS (a `witness_failure` row and an error), never a result.
--
-- host.args: root, mode (default control), scenario (lebanon_retail default | georgia_retail), case (27, the only case), out (JSON Lines path)

io.stdout:setvbuf ("no")

local huge = math.huge
local function finite (v) return type (v) == "number" and v == v and v ~= huge and v ~= -huge end

local args = host.args
local function refuse (message) error ("env-ambient-authority: " .. message, 0) end
local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local mode = args.mode or "control"
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario_name = args.scenario == nil and "lebanon_retail" or args.scenario
local scenario = scenarios [scenario_name]
if scenario == nil then refuse ("scenario must be lebanon_retail or georgia_retail, got '" .. tostring (scenario_name) .. "'") end
-- the selected characterized case of the fixture's group 11 (csrc/eech_env_fixture.c), by index and label: the one case with a single call of the routine and all six terrain children
local CASES = { [27] = "terrain_sea_rising_from_zero" }
local case = tonumber (args.case or "27")
if not case or CASES [case] == nil then refuse ("case must be 27, got '" .. tostring (args.case) .. "'") end
local seed = 1
local out = assert (io.open (out_path, "w"))

local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
local function json_string (s)
	return '"' .. (string.gsub (s, '[%c"\\\128-\255]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)) .. '"'
end
-- every write and flush of the evidence is checked: a failed one is a failure of the witness, even after the last row
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

-- ---------------------------------------------------------------------------------------------------------------- the modes
-- the valid distinct dyadic answers (all exact in single precision): the environment's four levels (the wind levels within 0 to 1), the terrain's i-th amplitude (4 i + 1) / 256
local SENT_ENV = { rain_light = 0.375, rain_heavy = 0.1875, wind_light = 0.625, wind_heavy = 0.3125 }
local function sent_terrain (count)
	local list = {}
	for i = 1, count do list [i] = (4 * i + 1) / 256 end
	return list
end
local MODES = {
	control = {},
	changed_env_ambient = { env = function () return { rain_light = SENT_ENV.rain_light, rain_heavy = SENT_ENV.rain_heavy, wind_light = SENT_ENV.wind_light, wind_heavy = SENT_ENV.wind_heavy } end },
	changed_terrain_ambient = { terrain = function (count) return { amplitudes = sent_terrain (count) } end },
	withheld_env_ambient = { env = function () error ("the host withholds its ambient environment answer", 0) end, marker = "[env-provider:ambient.callback]" },
	malformed_env_ambient = { env = function () return { rain_light = 0.375, rain_heavy = 0.1875, wind_light = 1.5, wind_heavy = 0.3125 } end, marker = "[env-provider:ambient.answer]" },
	withheld_terrain_ambient = { terrain = function () error ("the host withholds its ambient terrain answer", 0) end, marker = "[terrain-provider:ambient.callback]" },
	malformed_terrain_ambient = { terrain = function (count) return { amplitudes = sent_terrain (count - 1) } end, marker = "[terrain-provider:ambient.amplitudes]" },
}
local plan = MODES [mode]
if plan == nil then refuse ("unknown mode '" .. tostring (mode) .. "'") end

-- ---------------------------------------------------------------------------------------------------------------- the host's providers, counted; only op = "ambient" altered, only while armed
local function g (v) if type (v) == "number" then return string.format ("%.17g", v) end return tostring (v) end
local factory_env = assert (host.env_provider, "env-ambient-authority: this host has no host.env_provider")
local factory_terrain = assert (host.terrain_provider, "env-ambient-authority: this host has no host.terrain_provider")
local map = scenario.map
local real_env, real_terrain = factory_env (), factory_terrain (root, map)
local armed = false
local requests = { env = 0, terrain = 0 }
local altered = { env = 0, terrain = 0 }
local ROW_LIMIT = 4
local rows_written = { env = 0, terrain = 0 }
local ambient_rows = {}
local function record (row, provider)
	if rows_written [provider] < ROW_LIMIT then
		rows_written [provider] = rows_written [provider] + 1
		ambient_rows [#ambient_rows + 1] = row
	end
end
local function environment (request)
	if not (armed and request.op == "ambient") then return real_env (request) end
	requests.env = requests.env + 1
	local n, ok, reply = requests.env, true, nil
	local sent = plan.env ~= nil
	if sent then
		altered.env = altered.env + 1
		ok, reply = pcall (plan.env)
	else
		reply = real_env (request)
	end
	local row = { event = "ambient", provider = "env", n = n, sent = sent }
	if ok and type (reply) == "table" then
		row.rain_light, row.rain_heavy, row.wind_light, row.wind_heavy = g (reply.rain_light), g (reply.rain_heavy), g (reply.wind_light), g (reply.wind_heavy)
	else
		row.rain_light, row.rain_heavy, row.wind_light, row.wind_heavy = "withheld", "withheld", "withheld", "withheld"
	end
	record (row, "env")
	if not ok then error (reply, 0) end
	return reply
end
local function terrain (request)
	if not (armed and request.op == "ambient") then return real_terrain (request) end
	requests.terrain = requests.terrain + 1
	-- the count and the classes asked about are read BEFORE the reply is built
	local count = #request.children
	local classes = {}
	for i, child in ipairs (request.children) do classes [i] = tostring (child.class) end
	local n, ok, reply = requests.terrain, true, nil
	local sent = plan.terrain ~= nil
	if sent then
		altered.terrain = altered.terrain + 1
		ok, reply = pcall (plan.terrain, count)
	else
		reply = real_terrain (request)
	end
	local row = { event = "ambient", provider = "terrain", n = n, sent = sent, count = count, classes = table.concat (classes, ",") }
	if ok and type (reply) == "table" and type (reply.amplitudes) == "table" then
		local list = {}
		for i, value in ipairs (reply.amplitudes) do list [i] = g (value) end
		row.amplitudes = table.concat (list, ",")
	else
		row.amplitudes = "withheld"
	end
	record (row, "terrain")
	if not ok then error (reply, 0) end
	return reply
end

-- ---------------------------------------------------------------------------------------------------------------- boot (no observer: it is not needed for this witness)
local dc = require ("eech_dc")
dc.prepare_installation (root)
host.use_reference_arithmetic () -- the reference host's arithmetic profile, selected right after the installation is prepared (src/reference_arithmetic.rs)
local engine = dc.boot {
	install_root = root, map = map, terrain = terrain, environment = environment, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache", seed = seed,
	bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss, supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = host.model,
	sense = host.sense, aim = host.aim,
}
emit { event = "boot", mode = mode, scenario = scenario_name, case = case, seed = seed, feature = "env-fixture" }

local present, method = pcall (function () return engine.env_case end)
if not present or type (method) ~= "function" then
	emit { event = "refusal", reason = "env_case_unavailable", detail = "the module was built without the env-fixture feature" }
	refuse ("the engine has no env_case method: build eech-dc with the env-fixture feature")
end

-- ---------------------------------------------------------------------------------------------------------------- the fixture's text (group 11), read as the characterization driver reads it
local function first_line (text) return (string.match (tostring (text), "[^\r\n]+") or tostring (text)) end
local function fields (line)
	local t = {}
	for k, v in string.gmatch (line, "(%S+)=(%S+)") do t [k] = v end
	return t
end
local function lines_of (text)
	local lines = {}
	for line in string.gmatch (text, "[^\n]+") do lines [#lines + 1] = line end
	return lines
end
local function hex_exact (value, digits) return type (value) == "string" and #value == digits and string.find (value, "^[0-9a-f]+$") ~= nil end
local function bool_fact (value) return value == "0" or value == "1" end
-- `ordinal:sub_type:class:word,...`: a non-empty list, the word is eight lowercase hex digits (these records carry no entity id)
local function children_of (value, what)
	local list = {}
	if value == nil then hard (what .. " has no children field") end
	if value == "" or string.find (value, "^,") or string.find (value, ",$") or string.find (value, ",,", 1, true) then hard (what .. " is not a comma separated child list: '" .. value .. "'") end
	for entry in string.gmatch (value, "[^,]+") do
		local idx, sub, name, word = string.match (entry, "^(%d+):(%d+):([%w_]+):(%x+)$")
		if not idx or not hex_exact (word, 8) then hard (what .. " has a malformed child entry '" .. entry .. "' (ordinal:sub type:class:eight lowercase hex digits)") end
		list [#list + 1] = { idx = tonumber (idx), sub = tonumber (sub), name = name, word = word }
	end
	if #list == 0 then hard (what .. " lists no children") end
	return list
end
-- the slots of one vector are the consecutive ordinals 0, 1, 2 ...; two vectors of the same session list agree on the sub type and the class of every slot (only the amplification words may differ)
local function consecutive (list, what)
	for i, child in ipairs (list) do
		if child.idx ~= i - 1 then hard (what .. " has the ordinal " .. child.idx .. " in slot " .. (i - 1) .. ": the ordinals are not consecutive from 0") end
	end
end
local function same_slots (a, b, what)
	if #a ~= #b then hard (what .. " lists " .. #b .. " children for " .. #a) end
	for i = 1, #a do
		if a [i].sub ~= b [i].sub or a [i].name ~= b [i].name then
			hard (what .. " slot " .. (i - 1) .. " is " .. b [i].sub .. ":" .. b [i].name .. ", not the " .. a [i].sub .. ":" .. a [i].name .. " listed before the call")
		end
	end
end
local function ask (index)
	local ok, text = pcall (engine.env_case, engine, 11, index)
	if not ok then return nil, first_line (text) end
	if string.find (text, "env-case-error", 1, true) then hard ("env_case(11, " .. index .. ") refused: " .. text) end
	return lines_of (text)
end

local plan_lines = ask (0)
if not plan_lines then hard ("env_case(11, 0) failed") end
local p = fields (plan_lines [1] or "")
local cases, child_count = tonumber (p.cases), tonumber (p.children)
if not string.find (plan_lines [1] or "", "^env%-sound%-plan version=1 ") or p.camera ~= "1" or p.complete ~= "1" or not cases or cases ~= math.floor (cases) or cases < case or cases > 64 or child_count ~= 10 then
	hard ("the plan is not a version 1 group 11 plan with a camera, the ten ambient children and the selected case: " .. tostring (plan_lines [1]))
end
emit { event = "plan", raw = plan_lines [1], cases = cases, children = child_count }

-- the selected case, with the providers armed JUST before it runs
armed = true
local lines, failure = ask (case)
armed = false
local case_row
if lines then
	local head = lines [1] or ""
	local f = fields (head)
	if not string.find (head, "^env%-sound kind=original ") or tonumber (f.i) ~= case or f.label ~= CASES [case] then hard ("the selected case " .. case .. " is not the group 11 case '" .. CASES [case] .. "': " .. head) end
	if f.refused ~= nil then hard ("CONTROL NOT RUN: the fixture refused the selected case: " .. head) end
	local last = lines [#lines] or ""
	local e = fields (last)
	if not string.find (last, "^env%-sound%-end ") or tonumber (e.i) ~= case then hard ("the selected case does not end with its env-sound-end row: " .. last) end
	-- case 27 is a ONE-call case: the routine ran once, so the host was asked once by each provider
	local ncalls = tonumber (f.calls)
	if ncalls ~= 1 then hard ("the selected case is a one-call case, not " .. tostring (f.calls) .. " calls: " .. head) end
	local call_rows, session_rows = {}, {}
	for i = 2, #lines - 1 do
		if string.find (lines [i], "^env%-sound%-call ") then call_rows [#call_rows + 1] = lines [i]
		elseif string.find (lines [i], "^env%-sound%-session ") then session_rows [#session_rows + 1] = lines [i]
		else hard ("the selected case has an unexpected row: " .. lines [i]) end
	end
	if #call_rows ~= ncalls or #session_rows ~= 1 then hard ("the selected case has " .. #call_rows .. " call rows for " .. ncalls .. " calls and " .. #session_rows .. " session rows") end
	-- the session row: of THIS case, with the original's 0/1 fact (a 0 is a fact about the original, kept in the raw record and never repaired)
	local session_row = session_rows [1]
	local s = fields (session_row)
	if not string.find (s.i or "", "^%d+$") or tonumber (s.i) ~= case then hard ("the selected case has a session row of another case or without its index: " .. session_row) end
	if not bool_fact (s.session_unchanged_by_the_routine) then hard ("the selected case has a session row without a 0 or 1 fact: " .. session_row) end
	-- the vector before the call, in the real list order (head insertion): every child at the sentinel (rain, wind) or at the case's start value (terrain) the fixture says it set
	local before = children_of (f.children_before, "the case's children_before")
	consecutive (before, "the case's children_before")
	if #before ~= child_count then hard ("the case lists " .. #before .. " children before the call, the plan " .. child_count) end
	if not hex_exact (f.amp0_w, 8) then hard ("FIXTURE DEFECT: the selected case has no eight-digit amp0_w: " .. head) end
	for _, child in ipairs (before) do
		local rain_wind = child.name == "light_rain" or child.name == "heavy_rain" or child.name == "light_wind" or child.name == "heavy_wind"
		if child.word ~= (rain_wind and "3f200000" or f.amp0_w) then hard ("FIXTURE DEFECT: child " .. child.name .. " started at " .. child.word .. ": " .. head) end
	end
	local after
	for n, row in ipairs (call_rows) do
		local r = fields (row)
		if tonumber (r.i) ~= case or tonumber (r.call) ~= n then hard ("call row " .. n .. " is not of the selected case: " .. row) end
		if not bool_fact (r.rng_unchanged) then hard ("call row " .. n .. " has no 0 or 1 rng_unchanged fact: " .. row) end
		after = children_of (r.children, "call " .. n)
		consecutive (after, "call " .. n)
		same_slots (before, after, "call " .. n)
	end
	-- the fixture restored what it says it restored (a defect of the fixture, never a result of the host)
	for _, name in ipairs { "restored", "children_restored", "scratch_restored", "camera_restored", "cloud_restored", "in_cockpit_restored" } do
		if e [name] ~= "1" then hard ("FIXTURE DEFECT: " .. name .. " is not 1 after the selected case: " .. last) end
	end
	if not hex_exact (e.fp_before, 16) or not hex_exact (e.fp_after, 16) or e.fp_before ~= e.fp_after then hard ("FIXTURE DEFECT: the state fingerprints of the selected case are not two equal 16-digit words: " .. last) end
	case_row = { event = "case", i = case, label = f.label, calls = ncalls, children_before = f.children_before, children = fields (call_rows [#call_rows]).children, restored = e.restored, raw = table.concat (lines, "\n") }
end

for _, row in ipairs (ambient_rows) do emit (row) end
if case_row then emit (case_row) end
emit {
	event = "authority", mode = mode, expected_marker = plan.marker or "", completed = lines ~= nil, failure = failure or "", case = case,
	env_ambient_requests = requests.env, terrain_ambient_requests = requests.terrain, env_altered = altered.env, terrain_altered = altered.terrain,
	request_rows_env = rows_written.env, request_rows_terrain = rows_written.terrain, calls = case_row and case_row.calls or -1,
}
-- the last thing the witness does is close its evidence, and a close that fails is a failure after the success row as much as before it
local closed, close_error = out:close ()
if not closed then refuse ("OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error)) end
io.stderr:write (string.format ("env-ambient-authority: mode=%s case=%d completed=%s env_requests=%d terrain_requests=%d env_altered=%d terrain_altered=%d failure=%s\n", mode, case, tostring (lines ~= nil),
	requests.env, requests.terrain, altered.env, altered.terrain, failure or "none"))
