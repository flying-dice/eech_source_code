-- TEST-ONLY DRIVER of the ORIGINAL helicopter rotor sound computation (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
--
-- WHAT IT ASKS. The ORIGINAL aphavoc/source/entity/mobile/aircraft/helicop/helicop.c update_local_helicopter_rotor_sounds, which hc_updt.c calls for every helicopter (the AI ones too) on the supported campaign path
-- and which needs no backend, playing handle or device: from the helicopter's stored main rotor rpm and player kind (and, for the local player, the flight model's acceleration vector and engine words) it computes the rotor,
-- slap, turbine and APU levels and pitches, asks the sound children to pause or resume, and writes the levels and pitches into the sound effect children. Nothing here moves or changes it. This driver asks WHAT the
-- original does, from the ROTOR PROBE (ss_envfx.h ENV_FXR; csrc/eech_env_observer.c), one record per call, complete enough for a later independent bit-for-bit replay of the function from the record alone: the inputs it
-- reads (as IEEE words), the ORDERED pause and resume requests with the sound sub-type and delay, every special-effect child it walks with the sound effect's stored amplification and pitch before and after, and the
-- shared seed word on entry and exit (so a draw by any helper inside the call is visible; the probe draws nothing).
--
-- MEASURED vs INFERRED. Everything in a record is MEASURED by the probe in the original code. What this driver adds is only counting and structural checking of the records, labelled `derived`; it computes no level, pitch or
-- acceleration and replays nothing. A branch that no natural call took is not covered by the natural capture: the controlled sweep exists for that and is labelled as world-input variation, never natural coverage.
-- Nothing here shows that any sound plays or is audible; no backend, device, voice or handle exists or is faked.
--
-- NATURAL CAPTURE. The probe is armed ONLY by `dc.env_rotor_arm (true)`, before boot, and by nothing else: the general observer is NOT armed, so the accepted event stream and every older report are untouched. The driver runs
-- `seconds` of natural frames and DRAINS the probe after boot and after every frame into the raw sidecar `<out>.rotor.raw` (the native text, written and flushed BEFORE it is validated), so the probe's page (1024 calls) is
-- never exceeded in a frame unless the world is larger than that, in which case the probe says so and the driver REFUSES: nothing is truncated into a result.
--
-- THE SWEEP (`sweep=ai` or `sweep=fd`; default none). After the natural frames and after the read-only sound-initialisation report, the fixture case (group 14, index 16 or 17) runs the original function on one real
-- helicopter with chosen stored inputs, and the probe records those calls stamped `phase=1`. It is PROCESS-ISOLATED and MUTATING (declared): it restores only the helicopter's rpm, player kind and the flight dynamics
-- pointer; it does not restore the sound children, which is why it runs last and says so. `ai` sweeps the stored rpm over the 30 and 45 thresholds and their neighbours with the helicopter's own AI player kind and no flight
-- model (the natural headless branch). `fd` gives the helicopter the local player kind and an OWNED flight dynamics record (the branch no headless run reaches; optional and separate for that reason). A refused case
-- is a failure of the witness.
--
-- TRANSPARENCY. After the natural frames the driver also keeps the existing read-only sound-initialisation report (group 14, index 15) in `<out>.sndinit.raw` and copies its fingerprint and constructor rows into the
-- output unvalidated: compared with the same report of an UNARMED run of the same scenario it shows whether arming the rotor probe changed the shared seed word, the entity indices or the constructor reads. The accepted
-- 17283-row observation-off stream is compared by its own driver, with the probe unarmed.
--
-- FAILURES (no result row; `witness_failure` or `refusal` and an error): a module without the rotor functions, a failed drain or frame, a malformed or incomplete record, a sequence gap, a probe counter that is not zero
-- (dropped, over_actions, over_children, nested, orphans, bad_points, pending, open), a calls total that is not the lines read, a refused or inconsistent sweep, and any failed write, flush or close of the evidence.
--
-- ARGUMENTS (host.args):
--   eech-world <this script> root=<installation root> out=<JSON Lines path> [scenario=lebanon_retail|georgia_retail] [seconds=<0..600; default 0>] [frame_ms=<10..1000; default 100>] [sweep=none|ai|fd]
--   Evidence files, all created EXCLUSIVELY through host.output_create_new (never a truncating open): `out`, `<out>.rotor.raw`, `<out>.sndinit.raw` and, with a sweep, `<out>.sweep.raw`.

io.stdout:setvbuf ("no")

local args = host.args
local function refuse (message) error ("env-rotor-sounds: " .. message, 0) end

local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario_name = args.scenario == nil and "lebanon_retail" or args.scenario
local scenario = scenarios [scenario_name]
if scenario == nil then refuse ("scenario must be lebanon_retail or georgia_retail, got '" .. tostring (scenario_name) .. "'") end
local seconds = args.seconds == nil and 0 or tonumber (args.seconds)
if seconds == nil or seconds ~= math.floor (seconds) or seconds < 0 or seconds > 600 then refuse ("seconds must be an integer from 0 to 600, got '" .. tostring (args.seconds) .. "'") end
local frame_ms = args.frame_ms == nil and 100 or tonumber (args.frame_ms)
if frame_ms == nil or frame_ms ~= math.floor (frame_ms) or frame_ms < 10 or frame_ms > 1000 then refuse ("frame_ms must be an integer from 10 to 1000, got '" .. tostring (args.frame_ms) .. "'") end
local sweep = args.sweep == nil and "none" or args.sweep
if sweep ~= "none" and sweep ~= "ai" and sweep ~= "fd" then refuse ("sweep must be none, ai or fd, got '" .. tostring (args.sweep) .. "'") end
local seed = 1

-- every evidence file is created exclusively, here, before anything is booted
local create = host.output_create_new
if type (create) ~= "function" then refuse ("this host has no host.output_create_new: evidence files cannot be created exclusively, and the driver does not fall back to a truncating open") end
local paths = { out = out_path, rotor = out_path .. ".rotor.raw", sndinit = out_path .. ".sndinit.raw" }
if sweep ~= "none" then paths.sweep = out_path .. ".sweep.raw" end
local files, created = {}, {}
for _, name in ipairs ({ "out", "rotor", "sndinit", "sweep" }) do
	if paths [name] ~= nil then
		local file, why = create (paths [name])
		if file == nil then
			for _, done in ipairs (created) do pcall (files [done].close, files [done]) end
			refuse ("OUTPUT FAILURE: creating " .. paths [name] .. " failed: " .. tostring (why) .. " (evidence is created exclusively and never overwritten; the same file under two names is refused)")
		end
		files [name] = file
		created [#created + 1] = name
	end
end
local out = files.out

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
local function first_line (text) return string.match (tostring (text), "[^\r\n]+") or tostring (text) end
-- the native text, kept whole BEFORE any of it is validated
local function keep (name, text)
	local file = files [name]
	local ok, err = file:write (text)
	if not ok then refuse ("OUTPUT FAILURE: writing " .. paths [name] .. " failed: " .. tostring (err)) end
	ok, err = file:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. paths [name] .. " failed: " .. tostring (err)) end
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
-- the hooks exist only in a module built with the env-observer feature; ONLY the rotor probe is armed, before boot, and the general observer is NOT
for _, name in ipairs ({ "env_rotor_arm" }) do
	if type (dc [name]) ~= "function" then
		emit { event = "refusal", reason = "rotor_probe_unavailable", detail = "the module was built without the env-observer feature" }
		refuse ("the module has no " .. name .. " function: build eech-dc with the env-observer feature")
	end
end
dc.env_rotor_arm (true)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign,
	gunship = "apache", seed = seed, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, seed = seed, seconds = seconds, frame_ms = frame_ms, sweep = sweep, feature = "env-observer", probe = "rotor", armed_by = "dc.env_rotor_arm only" }
for _, name in ipairs ({ "env_rotor_events", "env_rotor_report", "env_case" }) do
	local present, method = pcall (function () return engine [name] end)
	if not present or type (method) ~= "function" then
		emit { event = "refusal", reason = name .. "_unavailable", detail = "the module was built without the env-observer and env-fixture features" }
		refuse ("the engine has no " .. name .. " method: build eech-dc with the env-observer and env-fixture features")
	end
end

-- ---------------------------------------------------------------------------------------------------------------- the probe's report: its counters and its stated domain limits
local function report ()
	local ok, text = pcall (engine.env_rotor_report, engine)
	if not ok then hard ("the rotor probe's report FAILED in the engine: " .. first_line (text)) end
	if type (text) ~= "string" or string.sub (text, -1) ~= "\n" then hard ("the rotor probe's report is empty or truncated") end
	local line = string.match (text, "^([^\n]*)\n$")
	if line == nil or string.sub (line, 1, #"env-rotor-report ") ~= "env-rotor-report " then hard ("the rotor probe's report is not one env-rotor-report row: " .. text) end
	local map = {}
	for key, value in string.gmatch (line, "([%w_]+)=(%S+)") do
		if map [key] ~= nil then hard ("the rotor probe's report repeats the field " .. key .. ": " .. line) end
		map [key] = tonumber (value)
		if map [key] == nil or map [key] < 0 or map [key] ~= math.floor (map [key]) then hard ("the rotor probe's report field " .. key .. " is not a count: " .. line) end
	end
	for _, key in ipairs ({ "version", "armed", "open", "phase", "capacity", "max_actions", "max_children", "calls_total", "drained", "pending", "dropped", "over_actions", "over_children", "nested", "orphans", "bad_points" }) do
		if map [key] == nil then hard ("the rotor probe's report has no " .. key .. ": " .. line) end
	end
	if map.version ~= 1 then hard ("the rotor probe's report has another version: " .. line) end
	return map, line
end
-- a capture is complete only if the probe lost nothing and left nothing open or pending
local function require_clean (map, line, when)
	if map.armed ~= 1 then hard ("the rotor probe is not armed " .. when .. ": " .. line) end
	for _, key in ipairs ({ "dropped", "over_actions", "over_children", "nested", "orphans", "bad_points", "pending", "open" }) do
		if map [key] ~= 0 then hard ("the rotor probe's " .. key .. " is " .. map [key] .. " " .. when .. ": the capture is incomplete (a call was lost, truncated, nested or left open): " .. line) end
	end
end
local limits, limits_line = report ()
emit { event = "rotor_limits", raw = limits_line, capacity = limits.capacity, max_actions = limits.max_actions, max_children = limits.max_children }

-- ---------------------------------------------------------------------------------------------------------------- the records
local FIELDS = { "seq", "t", "phase", "ent", "player", "fd", "rng_in", "rng_out", "rpm_w", "acc_x_w", "acc_y_w", "acc_z_w", "lrpm_w", "rrpm_w", "apu_w", "actions", "visited", "sound", "act", "kids" }
local WORDS = { rpm_w = true, acc_x_w = true, acc_y_w = true, acc_z_w = true, lrpm_w = true, rrpm_w = true, apu_w = true }
local totals = { calls = 0, bytes = 0, phase0 = 0, phase1 = 0, fd_calls = 0, with_actions = 0, rng_changed = 0, kids_seen = 0 }
local entities, distinct = {}, 0
local last_seq = -1
local sweep_calls = 0
local function is_word (v) return type (v) == "string" and #v == 8 and string.find (v, "^%x+$") ~= nil end
local function check_call (line)
	local f = {}
	for key, value in string.gmatch (line, "([%w_]+)=(%S+)") do
		if f [key] ~= nil then hard ("a rotor call repeats the field " .. key .. ": " .. line) end
		f [key] = value
	end
	for _, key in ipairs (FIELDS) do
		if f [key] == nil then hard ("a rotor call has no field " .. key .. ": " .. line) end
	end
	local n = {}
	for _, key in ipairs (FIELDS) do
		if WORDS [key] then
			if not is_word (f [key]) then hard ("the rotor call field " .. key .. " is not an eight digit word: " .. line) end
		elseif key ~= "act" and key ~= "kids" then
			n [key] = tonumber (f [key])
			if n [key] == nil or n [key] ~= math.floor (n [key]) then hard ("the rotor call field " .. key .. " is not an integer: " .. line) end
		end
	end
	-- the sequence has no gap: a lost call would show as a missing number, whatever the counters say
	if n.seq ~= last_seq + 1 then hard ("the rotor calls are out of sequence (expected " .. (last_seq + 1) .. "): " .. line) end
	last_seq = n.seq
	if n.phase ~= 0 and n.phase ~= 1 then hard ("the rotor call phase is neither 0 nor 1: " .. line) end
	if n.fd ~= 0 and n.fd ~= 1 then hard ("the rotor call flight model flag is neither 0 nor 1: " .. line) end
	if n.ent < 0 or n.actions < 0 or n.visited < 0 or n.sound < 0 or n.sound > n.visited then hard ("the rotor call counts are not coherent: " .. line) end
	if n.fd == 0 then
		for key in pairs (WORDS) do
			if key ~= "rpm_w" and f [key] ~= "00000000" then hard ("a rotor call without a flight model carries a flight model word (" .. key .. "): " .. line) end
		end
	end
	-- the ordered requests: kind:sub:delay, at most max_actions listed (the probe refuses a call with more)
	local listed = math.min (n.actions, limits.max_actions)
	local count = 0
	if f.act == "-" then
		if listed ~= 0 then hard ("a rotor call lists no request where it records " .. n.actions .. ": " .. line) end
	else
		for kind, sub, delay in string.gmatch (f.act, "(%d+):(%d+):(%x+),?") do
			count = count + 1
			if (kind ~= "0" and kind ~= "1") or not is_word (delay) then hard ("a rotor request is malformed: " .. line) end
		end
		if count ~= listed or select (2, string.gsub (f.act, ":", "")) ~= 2 * listed then hard ("a rotor call lists " .. count .. " requests where it records " .. n.actions .. ": " .. line) end
	end
	-- the children walked: type:sub:before_amp:before_pitch:after_amp:after_pitch:after_seen
	listed = math.min (n.visited, limits.max_children)
	count = 0
	local sound_after = 0
	if f.kids == "-" then
		if listed ~= 0 then hard ("a rotor call lists no child where it records " .. n.visited .. ": " .. line) end
	else
		for type_, sub, ba, bp, aa, ap, seen in string.gmatch (f.kids, "(%d+):(%d+):(%x+):(%x+):(%x+):(%x+):(%d+),?") do
			count = count + 1
			if not (is_word (ba) and is_word (bp) and is_word (aa) and is_word (ap)) or (seen ~= "0" and seen ~= "1") then hard ("a rotor child is malformed: " .. line) end
			if seen == "1" then sound_after = sound_after + 1 end
		end
		if count ~= listed or select (2, string.gsub (f.kids, ":", "")) ~= 6 * listed then hard ("a rotor call lists " .. count .. " children where it records " .. n.visited .. ": " .. line) end
	end
	if sound_after > n.sound then hard ("a rotor call has more children seen after the switch than sound effects: " .. line) end
	totals.calls = totals.calls + 1
	totals [n.phase == 1 and "phase1" or "phase0"] = totals [n.phase == 1 and "phase1" or "phase0"] + 1
	if n.fd == 1 then totals.fd_calls = totals.fd_calls + 1 end
	if n.actions > 0 then totals.with_actions = totals.with_actions + 1 end
	if n.rng_in ~= n.rng_out then totals.rng_changed = totals.rng_changed + 1 end
	totals.kids_seen = totals.kids_seen + n.visited
	if not entities [n.ent] then entities [n.ent] = true; distinct = distinct + 1 end
	return n
end

local function drain (when)
	local ok, text = pcall (engine.env_rotor_events, engine)
	if not ok then hard ("draining the rotor probe " .. when .. " FAILED in the engine: " .. first_line (text)) end
	if type (text) ~= "string" then hard ("the rotor probe's events " .. when .. " are not text") end
	if text == "" then return 0 end
	-- kept whole BEFORE validation
	keep ("rotor", text)
	totals.bytes = totals.bytes + #text
	if string.sub (text, -1) ~= "\n" then hard ("the rotor events " .. when .. " do not end with a newline (a truncated page)") end
	local calls = 0
	for line in string.gmatch (text, "([^\n]*)\n") do
		if line == "" or string.find (line, "[^\32-\126]") then hard ("the rotor events " .. when .. " have an empty line or a byte that is not printable ASCII") end
		if string.sub (line, 1, #"env-rotor-call ") ~= "env-rotor-call " then hard ("a rotor event " .. when .. " is not an env-rotor-call row: " .. string.sub (line, 1, 80)) end
		local n = check_call (line)
		if n.phase == 1 then sweep_calls = sweep_calls + 1 end
		calls = calls + 1
	end
	return calls
end

-- the calls made during boot (the probe was armed before it) are read before the first frame
local boot_calls = drain ("after boot")
emit { event = "rotor_frame", frame = 0, calls = boot_calls }

-- ---------------------------------------------------------------------------------------------------------------- the natural frames
local frames = math.floor (seconds * 1000 / frame_ms)
for n = 1, frames do
	local ok, err = pcall (engine.frame, engine, frame_ms)
	if not ok then
		emit { event = "engine_failure", frame = n, reason = first_line (err) }
		refuse ("frame " .. n .. " FAILED in the engine: " .. first_line (err))
	end
	local calls = drain ("after frame " .. n)
	emit { event = "rotor_frame", frame = n, calls = calls }
end
local natural_map, natural_line = report ()
require_clean (natural_map, natural_line, "after the natural frames")
if natural_map.calls_total ~= totals.calls or natural_map.drained ~= totals.calls then hard ("the probe's calls (" .. natural_map.calls_total .. " total, " .. natural_map.drained .. " drained) are not the " .. totals.calls .. " lines read: " .. natural_line) end
emit { event = "rotor_report", when = "after_natural_frames", raw = natural_line }
local natural_calls = totals.calls

-- ---------------------------------------------------------------------------------------------------------------- the read-only sound-initialisation report, for the armed-versus-unarmed comparison
do
	local ok, text = pcall (engine.env_case, engine, 14, 15)
	if not ok then
		emit { event = "engine_failure", i = 15, reason = first_line (text) }
		refuse ("env_case (14, 15) FAILED in the engine: " .. first_line (text))
	end
	if type (text) ~= "string" or text == "" or string.find (text, "env-case-error", 1, true) then hard ("env_case (14, 15) did not give a report: " .. first_line (text)) end
	keep ("sndinit", text)
	local found = 0
	for _, prefix in ipairs ({ "env-sndinit-fingerprint ", "env-sndinit-ctor " }) do
		local row = string.match (text, "\n(" .. string.gsub (prefix, "%-", "%%-") .. "[^\n]*)") or (string.sub (text, 1, #prefix) == prefix and string.match (text, "^([^\n]*)"))
		if not row then hard ("the sound initialisation report has no " .. prefix .. "row") end
		found = found + 1
		emit { event = "sndinit_row", scope = "copied_unvalidated_for_the_armed_versus_unarmed_comparison", raw = row }
	end
end

-- ---------------------------------------------------------------------------------------------------------------- the controlled sweep (world-input variation, mutating, declared)
local sweep_expected
if sweep ~= "none" then
	local index = sweep == "ai" and 16 or 17
	local ok, text = pcall (engine.env_case, engine, 14, index)
	if not ok then
		emit { event = "engine_failure", i = index, reason = first_line (text) }
		refuse ("env_case (14, " .. index .. ") FAILED in the engine: " .. first_line (text))
	end
	if type (text) ~= "string" or text == "" or string.find (text, "env-case-error", 1, true) then hard ("env_case (14, " .. index .. ") did not give a report: " .. first_line (text)) end
	keep ("sweep", text)
	local refused = string.match (text, "env%-rotor%-refused reason=(%S+)")
	if refused then hard ("the controlled sweep was refused by the fixture (" .. refused .. "): it is a failure of the witness, never a result") end
	-- the declaration is read from the plan row itself: the end row also says mutating=1 and must not stand in for it
	local plan_row = string.match (text, "^(env%-rotor%-plan version=1 [^\n]*)\n")
	if not plan_row or not string.find (plan_row .. " ", " mutating=1 ", 1, true) or not string.find (plan_row .. " ", " process_isolated=required ", 1, true) then hard ("the controlled sweep does not declare itself mutating and process-isolated: " .. first_line (text)) end
	local rows = select (2, string.gsub (text, "env%-rotor%-sweep i=", ""))
	local ended = tonumber (string.match (text, "env%-rotor%-end refused=0 mutating=1 calls=(%d+)"))
	if ended == nil or ended ~= rows or rows < 1 then hard ("the controlled sweep reports " .. tostring (ended) .. " calls and " .. rows .. " sweep rows: " .. text) end
	sweep_expected = ended
	local calls = drain ("after the sweep")
	if sweep_calls ~= sweep_expected then hard ("the probe recorded " .. sweep_calls .. " phase 1 calls where the sweep made " .. sweep_expected) end
	emit { event = "rotor_frame", frame = frames + 1, calls = calls, scope = "controlled_sweep_" .. sweep }
	local map, line = report ()
	require_clean (map, line, "after the sweep")
	if map.calls_total ~= totals.calls or map.drained ~= totals.calls then hard ("the probe's calls (" .. map.calls_total .. " total, " .. map.drained .. " drained) are not the " .. totals.calls .. " lines read after the sweep: " .. line) end
	emit { event = "rotor_report", when = "after_sweep", raw = line }
end

-- ---------------------------------------------------------------------------------------------------------------- what the driver JOINS (labelled derived; never a measurement)
emit {
	event = "derived", source = "the driver's join of the records above; not a measurement",
	natural_calls = natural_calls, natural_phase0_calls = totals.phase0, controlled_phase1_calls = totals.phase1, calls = totals.calls, boot_calls = boot_calls, frames = frames,
	distinct_entities = distinct, calls_with_requests = totals.with_actions, calls_with_a_flight_model = totals.fd_calls, calls_where_the_seed_word_changed = totals.rng_changed, children_walked = totals.kids_seen,
	bytes = totals.bytes, sweep = sweep, last_seq = last_seq,
	coverage_note = "natural coverage is only the phase 0 calls; the phase 1 calls are controlled world input and are not natural coverage; no sound is claimed to play",
}
emit { event = "result", calls = totals.calls, natural_calls = natural_calls, sweep = sweep }

-- the last thing the witness does is close its evidence, and a close that fails is a failure after the last row as much as before it
for _, name in ipairs (created) do
	local closed, close_error = files [name]:close ()
	if not closed then refuse ("OUTPUT FAILURE: closing " .. paths [name] .. " failed: " .. tostring (close_error)) end
end
io.stderr:write (string.format ("env-rotor-sounds: calls=%d natural=%d sweep=%s\n", totals.calls, natural_calls, sweep))
