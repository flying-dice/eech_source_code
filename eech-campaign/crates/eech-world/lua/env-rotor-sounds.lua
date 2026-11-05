-- TEST-ONLY DRIVER of the ORIGINAL helicopter rotor sound computation (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
--
-- WHAT IT ASKS. The ORIGINAL aphavoc/source/entity/mobile/aircraft/helicop/helicop.c update_local_helicopter_rotor_sounds, which hc_updt.c calls for every helicopter (the AI ones too) on the supported campaign path
-- and which needs no backend, playing handle or device: from the helicopter's stored main rotor rpm and player kind (and, with a flight model, its acceleration vector and, for the LOCAL player only, its engine and
-- APU words) it computes the rotor, slap, turbine and APU levels and pitches, asks the sound children to pause or resume, and writes the levels and pitches into the sound effect children. Nothing here moves or changes it,
-- and it is characterised ATOP the already hosted baseline (weather, wind, session, lifetime, catalogue and backend exchanges): the build is never made with the host environment disabled. This driver asks WHAT the
-- original does, from the ROTOR PROBE (ss_envfx.h ENV_FXR; csrc/eech_env_observer.c), one record per call, complete enough for a later independent bit-for-bit replay of a DEFINED call from the record alone: the inputs
-- it consumes (as IEEE words), the ORDERED pause and resume requests with the sound sub-type and delay word, every special-effect child it walks with the sound effect's stored amplification and pitch before and after,
-- and the shared seed word on entry and exit (so a draw by any helper inside the call is visible; the probe draws nothing).
--
-- COPIED versus CONSUMED. The acceleration vector of a present flight model is consumed for EVERY player kind and is always copied; the left, right and APU engine words are consumed ONLY for the LOCAL player kind and are
-- copied only then (zero otherwise); the stored rpm and the player kind are always consumed and copied. The driver rejects a record that carries a word the original does not consume.
--
-- UNASSIGNED LOCALS AND THE CANONICAL BUILD CONTRACT. In the unchanged original, for a LOCAL player with a flight model, `right_turbine_amp` is assigned only when the engines are more than 1.0 apart and `apu_amp` only when the APU rpm is above 0,
-- yet the child switch reads them for any TURBINE2 or APU child. At the language level those reads are uninitialised and not functions of the recorded inputs, and nothing here initialises or changes the routine. The accepted canonical
-- whole-engine build, though, compiles every legacy unit and every csrc unit (helicop.c included) with `-ftrivial-auto-var-init=zero` (eech-engine-sys/build/main.rs, one cc::Build for all of them), so in the artifacts this driver
-- characterises those locals are zero by that EXISTING compile contract. The controlled sweep therefore executes EVERY shape; one whose unassigned local is actually read by a child of the chosen helicopter is labelled
-- `compiler_zero_init` (a compiler-compatibility case, reported apart from the safe-language cases, with its full output words and floating point rows), one whose unassigned local no child reads is `undefined_not_consumed`, the rest are
-- `defined`. The contract is DECLARED by the build source and by the plan row (`auto_var_init=declared_by_build_main_rs_zero`), not detected at run time: what the rows show is a measurement of that build only, the zero value is
-- PENDING Root's comparison and is never assumed here, and outside a build with that flag the language-level limitation stands. The natural headless branch (AI player, no flight model) assigns everything.
--
-- MEASURED vs INFERRED. Everything in a probe record is MEASURED by the probe in the original code. What this driver adds is counting, strict structural checking and JOINING of the records (each sweep call to the chosen helicopter,
-- case, player kind, flight model and declared input words; the probe's `after` words to the fixture's own output words), labelled `derived`; it computes no level, pitch or acceleration and replays nothing. A branch no natural call took
-- is not covered by the natural capture: the controlled sweep is world-input variation and is never natural coverage. Nothing here shows that any sound plays or is audible; no backend, device, voice or handle exists or is faked.
--
-- NATURAL CAPTURE. The probe is armed ONLY by `dc.env_rotor_arm (true)`, before boot, and by nothing else: the general observer is NOT armed, so the accepted event stream and every older report are untouched. The driver runs
-- `seconds` of natural frames and DRAINS the probe after boot and after every frame into the raw sidecar `<out>.rotor.raw` (the native text, written and flushed BEFORE it is validated). The page is 1024 calls; if the world is
-- larger the probe says so and the driver REFUSES: nothing is truncated into a result. A natural drain may carry ONLY phase 0 calls.
--
-- THE SWEEP (`sweep=ai` or `sweep=fd`; default none). After the natural frames and the read-only sound-initialisation report, the fixture case (group 14, index 16 or 17) runs the original function on one real helicopter with
-- chosen stored inputs, and the probe records those calls stamped `phase=1`; the sweep drain may carry ONLY phase 1 calls. It is PROCESS-ISOLATED and MUTATING (declared): it restores the helicopter's rpm, player kind, the flight
-- dynamics pointer and the probe's phase on every path, and does not restore the sound children, which is why it runs last. `ai` sweeps the stored rpm over the 30 and 45 thresholds and their neighbours with the helicopter's own AI
-- player kind and no flight model (the natural headless branch). `fd` gives the helicopter the local player kind and an OWNED flight dynamics record (the branch no headless run reaches; optional and separate for that reason). A refused
-- case is a failure of the witness. `short_buffer=<n>` first asks the same case for a report buffer of n bytes and requires the engine to REFUSE it before changing anything (the staging guard).
--
-- CLASS AND COHORT. The declared TURBINE2 and APU child counts of the sweep helicopter are joined to the real sub-types (TURBINE2 36, APU 34) of the sound children the output lists, and every flight model row's class is reconciled
-- with its declared engine and APU words (the original's assignment conditions on those finite words, not an amplitude or pitch oracle) and the consumed cohort, so a consumed unassigned local can never be relabelled as an ordinary
-- controlled input. The floating point rows are accepted only as genuine standard <fenv.h> observations of the x86 targets (rounding mode 0, 1024, 2048 or 3072, never negative or unknown; exception flags within 0x3F); that is
-- NOT the x87 control/status word or MXCSR and no proof of the full floating point state is claimed from it.
--
-- TRANSPARENCY (`probe=off` and the uninstrumented build). The sweep reports the OUTPUT WORDS each call left in the sound children (`env-rotor-out`) and the floating point rounding mode and exception flags around it
-- (`env-rotor-fp`), read by the fixture alone. The identical sweep on the same scenario in (a) a build WITHOUT the observer (no hook in the routine), (b) the observer build UNARMED (`probe=off`) and (c) the observer build ARMED
-- must print byte-identical `env-rotor-sweep`, `env-rotor-out` and `env-rotor-fp` rows: that comparison, not the seed, entity or constructor fingerprints, is what bears on the numerical transparency of the hooks. `probe=off` runs
-- the frames and the sweep only (no arm, drain or sound-initialisation report) so that it also works in a module built without the observer. The comparison is PENDING until Root runs it; the target is x86-64 with scalar SSE
-- float and double arithmetic (no x87 extended-precision temporaries), and the rounding mode is read before and after each call.
--
-- FAILURES (no result row; `witness_failure` or `refusal` and an error): a module without the rotor functions, a failed drain or frame, a malformed or incomplete record (the grammar is closed: exact fields, anchored lists, counts
-- within the limits, every sound child with exactly its stored after data), a sequence gap, a phase in the wrong drain, a probe counter that is not zero (dropped, over_actions, over_children, nested, orphans, bad_points,
-- pending, open), a calls total that is not the lines read, a refused, out-of-order or inconsistent sweep, a sweep call that does not join its declared case, helicopter and inputs, and any failed write, flush or close of the evidence.
--
-- ARGUMENTS (host.args):
--   eech-world <this script> root=<installation root> out=<JSON Lines path> [scenario=lebanon_retail|georgia_retail] [seconds=<0..600; default 0>] [frame_ms=<10..1000; default 100>] [sweep=none|ai|fd]
--                                                                       [probe=on|off] [short_buffer=<128..30719>]
--   Evidence files, all created EXCLUSIVELY through host.output_create_new (never a truncating open): `out`, with the probe `<out>.rotor.raw` and `<out>.sndinit.raw`, and with a sweep `<out>.sweep.raw`.

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
local probe_arg = args.probe == nil and "on" or args.probe
if probe_arg ~= "on" and probe_arg ~= "off" then refuse ("probe must be on or off, got '" .. tostring (args.probe) .. "'") end
local probe = probe_arg == "on"
local short_buffer
if args.short_buffer ~= nil then
	short_buffer = tonumber (args.short_buffer)
	if sweep == "none" or short_buffer == nil or short_buffer ~= math.floor (short_buffer) or short_buffer < 128 or short_buffer > 30719 then refuse ("short_buffer needs a sweep and an integer from 128 to 30719, got '" .. tostring (args.short_buffer) .. "'") end
end
local seed = 1

-- every evidence file is created exclusively, here, before anything is booted
local create = host.output_create_new
if type (create) ~= "function" then refuse ("this host has no host.output_create_new: evidence files cannot be created exclusively, and the driver does not fall back to a truncating open") end
local paths = { out = out_path }
if probe then
	paths.rotor = out_path .. ".rotor.raw"
	paths.sndinit = out_path .. ".sndinit.raw"
end
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
host.use_reference_arithmetic () -- the reference host's arithmetic profile, selected right after the installation is prepared (src/reference_arithmetic.rs)
-- the hooks exist only in a module built with the env-observer feature; ONLY the rotor probe is armed, before boot, and the general observer is NOT
if probe then
	if type (dc.env_rotor_arm) ~= "function" then
		emit { event = "refusal", reason = "rotor_probe_unavailable", detail = "the module was built without the env-observer feature" }
		refuse ("the module has no env_rotor_arm function: build eech-dc with the env-observer feature")
	end
	dc.env_rotor_arm (true)
end
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign,
	gunship = "apache", seed = seed, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = host.model, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, seed = seed, seconds = seconds, frame_ms = frame_ms, sweep = sweep, probe = probe_arg, feature = probe and "env-observer" or "none-required", armed_by = probe and "dc.env_rotor_arm only" or "nothing" }
local needed = { "env_case" }
if probe then needed = { "env_rotor_events", "env_rotor_report", "env_case" } end
if short_buffer then needed [#needed + 1] = "env_case_capacity" end
for _, name in ipairs (needed) do
	local present, method = pcall (function () return engine [name] end)
	if not present or type (method) ~= "function" then
		emit { event = "refusal", reason = name .. "_unavailable", detail = "the module was built without the env-observer and env-fixture features" }
		refuse ("the engine has no " .. name .. " method: build eech-dc with the env-observer and env-fixture features")
	end
end

-- ---------------------------------------------------------------------------------------------------------------- exact row grammar (used for the fixture's own rows)
local function exact (line, event, keys, what)
	local tokens = {}
	for token in string.gmatch (line, "%S+") do tokens [#tokens + 1] = token end
	if tokens [1] ~= event then hard (what .. ": expected a " .. event .. " row: " .. line) end
	if #tokens ~= #keys + 1 then hard (what .. ": the " .. event .. " row has " .. (#tokens - 1) .. " fields, the protocol has " .. #keys .. ": " .. line) end
	local map = {}
	for position, key in ipairs (keys) do
		local k, v = string.match (tokens [position + 1], "^([%w_]+)=([^=]+)$")
		if k ~= key then hard (what .. ": field " .. position .. " of the " .. event .. " row is not " .. key .. ": " .. line) end
		map [key] = v
	end
	return map
end
local function is_word (v) return type (v) == "string" and #v == 8 and string.find (v, "^%x+$") ~= nil end
local function as_count (v, line)
	local n = tonumber (v)
	if type (v) ~= "string" or string.find (v, "^%d+$") == nil or #v > 10 or n == nil then hard ("a count is not a plain non-negative integer (" .. tostring (v) .. "): " .. line) end
	return n
end
-- a comma separated list of tuples, each matching the anchored pattern, with no junk, no empty item and no trailing comma
local function items (list, pattern, line, what)
	local found = {}
	if list == "-" then return found end
	local rest = list .. ","
	while #rest > 0 do
		local item, tail = string.match (rest, "^([^,]*),(.*)$")
		if item == nil or item == "" then hard (what .. " list has an empty or malformed item: " .. line) end
		local parts = { string.match (item, pattern) }
		if #parts == 0 then hard (what .. " item '" .. item .. "' does not match its grammar: " .. line) end
		found [#found + 1] = parts
		rest = tail
	end
	return found
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
	if map.phase ~= 0 then hard ("the rotor probe's phase is " .. map.phase .. " " .. when .. " where every path must restore it to 0: " .. line) end
end
local limits, limits_line
if probe then
	limits, limits_line = report ()
	emit { event = "rotor_limits", raw = limits_line, capacity = limits.capacity, max_actions = limits.max_actions, max_children = limits.max_children }
end

-- ---------------------------------------------------------------------------------------------------------------- the records (a closed grammar)
local SOUND_EFFECT_TYPE = 32 -- ENTITY_TYPE_SOUND_EFFECT (en_types.h)
local PLAYER_AI, PLAYER_LOCAL, PLAYER_REMOTE = 0, 1, 2 -- ENTITY_PLAYERS (en_plyr.h)
local FIELDS = { "seq", "t", "phase", "ent", "player", "fd", "rng_in", "rng_out", "rpm_w", "acc_x_w", "acc_y_w", "acc_z_w", "lrpm_w", "rrpm_w", "apu_w", "actions", "visited", "sound", "act", "kids" }
local ZERO = "00000000"
local totals = { calls = 0, bytes = 0, phase0 = 0, phase1 = 0, fd_calls = 0, with_actions = 0, rng_changed = 0, kids_seen = 0 }
local entities, distinct = {}, 0
local last_seq = -1
local sweep_records = {}
local function check_call (line, expected_phase, when)
	local tokens = {}
	for token in string.gmatch (line, "%S+") do tokens [#tokens + 1] = token end
	if tokens [1] ~= "env-rotor-call" or #tokens ~= #FIELDS + 1 then hard ("a rotor call does not have exactly its " .. #FIELDS .. " fields: " .. line) end
	local f = {}
	for position, key in ipairs (FIELDS) do
		local k, v = string.match (tokens [position + 1], "^([%w_]+)=(%S+)$")
		if k ~= key then hard ("field " .. position .. " of a rotor call is not " .. key .. ": " .. line) end
		f [key] = v
	end
	local n = {}
	for _, key in ipairs (FIELDS) do
		if string.sub (key, -2) == "_w" then
			if not is_word (f [key]) then hard ("the rotor call field " .. key .. " is not an eight digit word: " .. line) end
		elseif key ~= "act" and key ~= "kids" then
			if string.find (f [key], "^-?%d+$") == nil or #f [key] > 11 then hard ("the rotor call field " .. key .. " is not a plain integer: " .. line) end
			n [key] = tonumber (f [key])
		end
	end
	-- the sequence has no gap: a lost call would show as a missing number, whatever the counters say
	if n.seq ~= last_seq + 1 then hard ("the rotor calls are out of sequence (expected " .. (last_seq + 1) .. "): " .. line) end
	last_seq = n.seq
	if n.phase ~= expected_phase then hard ("a phase " .. n.phase .. " rotor call arrived " .. when .. " where only phase " .. expected_phase .. " may: " .. line) end
	if n.fd ~= 0 and n.fd ~= 1 then hard ("the rotor call flight model flag is neither 0 nor 1: " .. line) end
	if n.player ~= PLAYER_AI and n.player ~= PLAYER_LOCAL and n.player ~= PLAYER_REMOTE then hard ("the rotor call player kind " .. n.player .. " is not an ENTITY_PLAYERS value: " .. line) end
	if n.ent < 0 or n.actions < 0 or n.visited < 0 or n.sound < 0 or n.sound > n.visited then hard ("the rotor call counts are not coherent: " .. line) end
	-- the counts are within the stated limits: a record beyond them is refused, never masked by a minimum
	if n.actions > limits.max_actions then hard ("a rotor call records " .. n.actions .. " requests beyond the limit of " .. limits.max_actions .. ": " .. line) end
	if n.visited > limits.max_children then hard ("a rotor call records " .. n.visited .. " children beyond the limit of " .. limits.max_children .. ": " .. line) end
	-- the words the original does not consume are zero: everything flight-model related without one; the engine and APU words unless the player is LOCAL
	for _, key in ipairs ({ "acc_x_w", "acc_y_w", "acc_z_w" }) do
		if n.fd == 0 and f [key] ~= ZERO then hard ("a rotor call without a flight model carries the word " .. key .. ": " .. line) end
	end
	for _, key in ipairs ({ "lrpm_w", "rrpm_w", "apu_w" }) do
		if (n.fd == 0 or n.player ~= PLAYER_LOCAL) and f [key] ~= ZERO then hard ("a rotor call carries the engine word " .. key .. " that the original consumes only for a LOCAL player with a flight model: " .. line) end
	end
	-- the ordered requests: kind:sub:delay, anchored and exhaustive, exactly as many as recorded
	local acts = items (f.act, "^([01]):(%d+):(%x%x%x%x%x%x%x%x)$", line, "a rotor request")
	if #acts ~= n.actions then hard ("a rotor call lists " .. #acts .. " requests where it records " .. n.actions .. ": " .. line) end
	-- the children walked: type:sub:before_amp:before_pitch:after_amp:after_pitch:after_seen, exactly as many as walked
	local kids = items (f.kids, "^(%d+):(%d+):(%x%x%x%x%x%x%x%x):(%x%x%x%x%x%x%x%x):(%x%x%x%x%x%x%x%x):(%x%x%x%x%x%x%x%x):([01])$", line, "a rotor child")
	if #kids ~= n.visited then hard ("a rotor call lists " .. #kids .. " children where it records " .. n.visited .. ": " .. line) end
	local sound_count = 0
	local sounds = {}
	for _, kid in ipairs (kids) do
		local kid_type, sub, ba, bp, aa, ap, seen = tonumber (kid [1]), tonumber (kid [2]), kid [3], kid [4], kid [5], kid [6], kid [7]
		if kid_type == SOUND_EFFECT_TYPE then
			sound_count = sound_count + 1
			if seen ~= "1" then hard ("a sound effect child of a rotor call has no stored after data (after_seen " .. seen .. "): " .. line) end
			if sub < 1 then hard ("a sound effect child of a rotor call has no sub-type: " .. line) end
			sounds [#sounds + 1] = { sub = sub, after_amp = aa, after_pitch = ap }
		else
			if sub ~= 0 or ba ~= ZERO or bp ~= ZERO or aa ~= ZERO or ap ~= ZERO or seen ~= "0" then hard ("a child that is not a sound effect (type " .. kid_type .. ") carries sound data: " .. line) end
		end
	end
	if sound_count ~= n.sound then hard ("a rotor call records " .. n.sound .. " sound effect children and lists " .. sound_count .. ": " .. line) end
	totals.calls = totals.calls + 1
	totals [n.phase == 1 and "phase1" or "phase0"] = totals [n.phase == 1 and "phase1" or "phase0"] + 1
	if n.fd == 1 then totals.fd_calls = totals.fd_calls + 1 end
	if n.actions > 0 then totals.with_actions = totals.with_actions + 1 end
	if n.rng_in ~= n.rng_out then totals.rng_changed = totals.rng_changed + 1 end
	totals.kids_seen = totals.kids_seen + n.visited
	if not entities [n.ent] then entities [n.ent] = true; distinct = distinct + 1 end
	return { n = n, f = f, sounds = sounds }
end

-- drains the probe; a natural drain (boot, frames) may carry ONLY phase 0 calls and the sweep drain ONLY phase 1
local function drain (when, expected_phase)
	local ok, text = pcall (engine.env_rotor_events, engine)
	if not ok then hard ("draining the rotor probe " .. when .. " FAILED in the engine: " .. first_line (text)) end
	if type (text) ~= "string" then hard ("the rotor probe's events " .. when .. " are not text") end
	if text == "" then return 0, {} end
	-- kept whole BEFORE validation
	keep ("rotor", text)
	totals.bytes = totals.bytes + #text
	if string.sub (text, -1) ~= "\n" then hard ("the rotor events " .. when .. " do not end with a newline (a truncated page)") end
	local calls, records = 0, {}
	for line in string.gmatch (text, "([^\n]*)\n") do
		if line == "" or string.find (line, "[^\32-\126]") then hard ("the rotor events " .. when .. " have an empty line or a byte that is not printable ASCII") end
		if string.sub (line, 1, #"env-rotor-call ") ~= "env-rotor-call " then hard ("a rotor event " .. when .. " is not an env-rotor-call row: " .. string.sub (line, 1, 80)) end
		records [#records + 1] = check_call (line, expected_phase, when)
		calls = calls + 1
	end
	return calls, records
end

-- ---------------------------------------------------------------------------------------------------------------- the calls made during boot, then the natural frames
local boot_calls = 0
if probe then
	boot_calls = drain ("after boot", 0)
	emit { event = "rotor_frame", frame = 0, calls = boot_calls }
end
local frames = math.floor (seconds * 1000 / frame_ms)
for n = 1, frames do
	local ok, err = pcall (engine.frame, engine, frame_ms)
	if not ok then
		emit { event = "engine_failure", frame = n, reason = first_line (err) }
		refuse ("frame " .. n .. " FAILED in the engine: " .. first_line (err))
	end
	if probe then
		local calls = drain ("after frame " .. n, 0)
		emit { event = "rotor_frame", frame = n, calls = calls }
	end
end
local natural_calls = totals.calls
if probe then
	local natural_map, natural_line = report ()
	require_clean (natural_map, natural_line, "after the natural frames")
	if natural_map.calls_total ~= totals.calls or natural_map.drained ~= totals.calls then hard ("the probe's calls (" .. natural_map.calls_total .. " total, " .. natural_map.drained .. " drained) are not the " .. totals.calls .. " lines read: " .. natural_line) end
	emit { event = "rotor_report", when = "after_natural_frames", raw = natural_line }

	-- the read-only sound-initialisation report, for the armed-versus-unarmed comparison
	local ok, text = pcall (engine.env_case, engine, 14, 15)
	if not ok then
		emit { event = "engine_failure", i = 15, reason = first_line (text) }
		refuse ("env_case (14, 15) FAILED in the engine: " .. first_line (text))
	end
	if type (text) ~= "string" or text == "" or string.find (text, "env-case-error", 1, true) then hard ("env_case (14, 15) did not give a report: " .. first_line (text)) end
	keep ("sndinit", text)
	for _, prefix in ipairs ({ "env-sndinit-fingerprint ", "env-sndinit-ctor " }) do
		local row = string.match (text, "\n(" .. string.gsub (prefix, "%-", "%%-") .. "[^\n]*)") or (string.sub (text, 1, #prefix) == prefix and string.match (text, "^([^\n]*)"))
		if not row then hard ("the sound initialisation report has no " .. prefix .. "row") end
		emit { event = "sndinit_row", scope = "copied_unvalidated_for_the_armed_versus_unarmed_comparison", raw = row }
	end
end

-- ---------------------------------------------------------------------------------------------------------------- the controlled sweep (world-input variation, mutating, declared)
-- real sound sub-types (en_sbtyp.h ENTITY_SUB_TYPE_EFFECTS): the cohort the declared child counts are joined to
local SUB_APU_TURBINE, SUB_ROTOR_TURBINE2 = 34, 36
-- the standard <fenv.h> observations of the x86 / x86-64 targets of this build: FE_TONEAREST 0, FE_DOWNWARD 0x400, FE_UPWARD 0x800, FE_TOWARDZERO 0xC00 (the engine entry sets round-toward-zero); the exception flags are the six
-- standard bits (0x3F). A negative (unavailable) or any other value is refused. This is NOT the x87 control/status word or MXCSR and no proof of the full floating point state is claimed from it.
local FENV_ROUND = { [0] = true, [1024] = true, [2048] = true, [3072] = true }
local FENV_EXCEPT_MASK = 63
-- an IEEE single word as its exact value (a double holds it exactly); nil for an infinity or a NaN
local function word_value (word)
	local n = tonumber (word, 16)
	local sign = 1
	if n >= 2147483648 then sign = -1; n = n - 2147483648 end
	local exponent = math.floor (n / 8388608)
	local mantissa = n - exponent * 8388608
	if exponent == 255 then return nil end
	if exponent == 0 then return sign * mantissa * 2 ^ -149 end
	return sign * (1 + mantissa / 8388608) * 2 ^ (exponent - 127)
end
-- the original's ASSIGNMENT CONDITIONS for the LOCAL player kind with a flight model, evaluated on the row's declared finite input words (not an amplitude or pitch oracle): right_turbine_amp is assigned only when the engines are
-- more than 1.0 apart, apu_amp only when the APU rpm is above 0. Returns whether each is left UNASSIGNED.
local function unassigned_locals (row, line)
	local left, right, apu = word_value (row.lrpm_w), word_value (row.rrpm_w), word_value (row.apu_w)
	if left == nil or right == nil or apu == nil then hard ("a sweep row declares an infinite or NaN engine word: " .. line) end
	return not (math.abs (left - right) > 1.0), not (apu > 0.0)
end
local function fenv_round (value, i, line)
	if type (value) ~= "string" or string.find (value, "^%d+$") == nil or #value > 5 or not FENV_ROUND [tonumber (value)] then
		hard ("the rounding mode '" .. tostring (value) .. "' of i=" .. i .. " is refused: it is not a standard x86 fenv value (0, 1024, 2048 or 3072; a negative, unavailable or unknown state is never accepted): " .. line)
	end
	return tonumber (value)
end

local executed, skipped_rows = {}, {}
local compiler_cases = 0
local sweep_entity
local fd_sweep = (sweep == "fd")
local function parse_sweep (text)
	if string.sub (text, -1) ~= "\n" then hard ("the sweep report does not end with a newline") end
	local lines = {}
	for line in string.gmatch (text, "([^\n]*)\n") do
		if line == "" or string.find (line, "[^\32-\126]") then hard ("the sweep report has an empty line or a byte that is not printable ASCII") end
		lines [#lines + 1] = line
	end
	local at = 1
	local what = "the sweep report"
	local function take (event, keys)
		local line = lines [at]
		if line == nil then hard (what .. " ends before its " .. event .. " row") end
		at = at + 1
		return exact (line, event, keys, what), line
	end
	if lines [1] ~= nil and string.sub (lines [1], 1, #"env-rotor-refused ") == "env-rotor-refused " then hard ("the controlled sweep was refused by the fixture (" .. lines [1] .. "): it is a failure of the witness, never a result") end
	local plan = take ("env-rotor-plan", { "version", "case", "process_isolated", "mutating", "natural", "phase", "restores", "children_restored", "instrumented", "probe_armed", "auto_var_init" })
	if plan.version ~= "1" or plan.process_isolated ~= "required" or plan.mutating ~= "1" or plan.natural ~= "0" or plan.phase ~= "1" or plan.children_restored ~= "0" then
		hard ("the controlled sweep does not declare itself mutating, process-isolated and not natural, or does not state that it restores no child: " .. lines [1])
	end
	if plan.case ~= (fd_sweep and "player_local_and_flight_model" or "ai_rpm_sweep") then hard ("the sweep report is for the case " .. plan.case .. " where " .. sweep .. " was asked: " .. lines [1]) end
	if plan.restores ~= "main_rotor_rpm,player,flight_dynamics_pointer,probe_phase" then hard ("the sweep does not declare the restoration it makes: " .. lines [1]) end
	if (plan.instrumented ~= "0" and plan.instrumented ~= "1") or (plan.probe_armed ~= "0" and plan.probe_armed ~= "1") then hard ("the sweep's instrumented and armed flags are not flags: " .. lines [1]) end
	if plan.auto_var_init ~= "declared_by_build_main_rs_zero" then hard ("the sweep does not state the build contract its unassigned locals rely on (auto_var_init): " .. lines [1]) end
	if probe and (plan.instrumented ~= "1" or plan.probe_armed ~= "1") then hard ("the rotor probe is on but the sweep reports instrumented " .. plan.instrumented .. " armed " .. plan.probe_armed .. ": " .. lines [1]) end
	local entity, entity_line = take ("env-rotor-entity", { "ent", "player", "special_effect_children", "sound_children", "turbine2_children", "apu_children", "fd" })
	local e = {}
	for key, value in pairs (entity) do e [key] = as_count (value, entity_line) end
	if e.player ~= PLAYER_AI or e.fd ~= 0 then hard ("the sweep helicopter must start as an AI helicopter without a flight model: " .. entity_line) end
	if e.sound_children < 1 or e.sound_children > e.special_effect_children or e.turbine2_children > e.sound_children or e.apu_children > e.sound_children then hard ("the sweep helicopter's child counts are not coherent: " .. entity_line) end
	if probe and e.special_effect_children > limits.max_children then hard ("the sweep helicopter has more children than the probe's per-call limit: " .. entity_line) end
	sweep_entity = e
	local index = 0
	while lines [at] ~= nil and string.sub (lines [at], 1, #"env-rotor-fingerprint ") ~= "env-rotor-fingerprint " do
		local words = { "i", "rpm_w", "ax_w", "ay_w", "az_w", "lrpm_w", "rrpm_w", "apu_w" }
		if string.sub (lines [at], 1, #"env-rotor-skipped ") == "env-rotor-skipped " then
			hard ("a skipped sweep row is not part of this protocol (every shape is executed under the build's zero auto-init contract): " .. lines [at])
		else
			local keys = {}
			for _, key in ipairs (words) do keys [#keys + 1] = key end
			keys [#keys + 1] = "class"
			local row, line = take ("env-rotor-sweep", keys)
			if as_count (row.i, line) ~= index then hard ("the sweep rows are out of order (expected i=" .. index .. "): " .. line) end
			for _, key in ipairs (words) do if key ~= "i" and not is_word (row [key]) then hard ("a sweep row word is not an eight digit word: " .. line) end end
			if row.class ~= "defined" and row.class ~= "undefined_not_consumed" and row.class ~= "compiler_zero_init" then hard ("a sweep row has an unknown class: " .. line) end
			if not fd_sweep then
				if row.class ~= "defined" then hard ("an AI sweep row is not fully defined: " .. line) end
				for _, key in ipairs ({ "ax_w", "ay_w", "az_w", "lrpm_w", "rrpm_w", "apu_w" }) do if row [key] ~= ZERO then hard ("an AI sweep row declares a flight model word: " .. line) end end
			end
			local out_row, out_line = take ("env-rotor-out", { "i", "kids" })
			if as_count (out_row.i, out_line) ~= index then hard ("the sweep output row does not belong to i=" .. index .. ": " .. out_line) end
			local kids = items (out_row.kids, "^(%d+):(%x%x%x%x%x%x%x%x):(%x%x%x%x%x%x%x%x)$", out_line, "a sweep output")
			if #kids ~= e.sound_children then hard ("the sweep output of i=" .. index .. " lists " .. #kids .. " sound children where the helicopter has " .. e.sound_children .. ": " .. out_line) end
			-- the declared TURBINE2 and APU child counts are the real sub-type cohort of the sound children the output lists
			local actual_t2, actual_apu = 0, 0
			for _, kid in ipairs (kids) do
				if tonumber (kid [1]) == SUB_ROTOR_TURBINE2 then actual_t2 = actual_t2 + 1 end
				if tonumber (kid [1]) == SUB_APU_TURBINE then actual_apu = actual_apu + 1 end
			end
			if actual_t2 ~= e.turbine2_children then hard ("the sweep helicopter declares " .. e.turbine2_children .. " TURBINE2 children but its sound outputs list " .. actual_t2 .. ": " .. out_line) end
			if actual_apu ~= e.apu_children then hard ("the sweep helicopter declares " .. e.apu_children .. " APU children but its sound outputs list " .. actual_apu .. ": " .. out_line) end
			-- the class is reconciled with the declared finite inputs AND the consumed cohort: a local the original leaves unassigned and a child reads is a compiler-compatibility case whatever the row is labelled
			if fd_sweep then
				local right_unassigned, apu_unassigned = unassigned_locals (row, line)
				local consumed = (right_unassigned and actual_t2 > 0) or (apu_unassigned and actual_apu > 0)
				local expected = consumed and "compiler_zero_init" or ((right_unassigned or apu_unassigned) and "undefined_not_consumed" or "defined")
				if row.class ~= expected then hard ("the sweep row i=" .. index .. " is labelled " .. row.class .. " but the classification gives " .. expected .. " (unassigned right_turbine_amp " .. tostring (right_unassigned) .. ", unassigned apu_amp " .. tostring (apu_unassigned) .. ", TURBINE2 children " .. actual_t2 .. ", APU children " .. actual_apu .. "): " .. line) end
			end
			local fp, fp_line = take ("env-rotor-fp", { "i", "round_before", "round_after", "excepts" })
			if as_count (fp.i, fp_line) ~= index then hard ("the sweep floating point row does not belong to i=" .. index .. ": " .. fp_line) end
			-- the rounding mode is a control the original function must leave alone (both are genuine standard fenv observations); the exception flags are an output, compared across builds
			local round_before, round_after = fenv_round (fp.round_before, index, fp_line), fenv_round (fp.round_after, index, fp_line)
			if round_before ~= round_after then hard ("the rounding mode changed across the original call of i=" .. index .. ": " .. fp_line) end
			if string.find (fp.excepts, "^%x%x?$") == nil or tonumber (fp.excepts, 16) > FENV_EXCEPT_MASK then hard ("the sweep exception flags '" .. fp.excepts .. "' of i=" .. index .. " are not a standard x86 exception flag set (the six bits 0x3F): " .. fp_line) end
			row.out = kids
			row.out_raw = out_row.kids
			row.fp = fp
			executed [#executed + 1] = row
		end
		index = index + 1
	end
	if index < 1 then hard ("the sweep report has no input row") end
	take ("env-rotor-fingerprint", { "rng_before_w", "rng_after_w", "entity_before", "entity_after", "local_before", "local_after" })
	local finish, finish_line = take ("env-rotor-end", { "refused", "mutating", "calls", "skipped" })
	if finish.refused ~= "0" or finish.mutating ~= "1" then hard ("the sweep end row is not an unrefused mutating end: " .. finish_line) end
	if as_count (finish.calls, finish_line) ~= #executed or as_count (finish.skipped, finish_line) ~= 0 or #executed < 1 then hard ("the sweep end row (" .. finish_line .. ") does not match the " .. #executed .. " executed rows and no skipped row") end
	if lines [at] ~= nil then hard ("the sweep report has a row after its end row: " .. lines [at]) end
	return plan
end

local sweep_calls = 0
if sweep ~= "none" then
	local index = fd_sweep and 17 or 16
	if short_buffer then
		-- the staged report is refused for a buffer that cannot hold it BEFORE anything is changed: the engine keeps running and the probe sees no call
		local before_map = probe and report ()
		local ok, err = pcall (engine.env_case_capacity, engine, 14, index, short_buffer)
		if ok then hard ("the engine accepted a report buffer of " .. short_buffer .. " bytes for the staged sweep") end
		if string.find (tostring (err), "rotor_case_refused_or_report_did_not_fit", 1, true) == nil then hard ("the short buffer was refused for another reason: " .. first_line (err)) end
		if probe then
			local after_map, after_line = report ()
			if after_map.calls_total ~= before_map.calls_total or after_map.phase ~= 0 or after_map.open ~= 0 or after_map.pending ~= 0 then hard ("the refused short-buffer sweep left the probe changed: " .. after_line) end
		end
		emit { event = "short_buffer_guard", capacity = short_buffer, reason = first_line (err) }
	end
	local ok, text = pcall (engine.env_case, engine, 14, index)
	if not ok then
		emit { event = "engine_failure", i = index, reason = first_line (text) }
		refuse ("env_case (14, " .. index .. ") FAILED in the engine: " .. first_line (text))
	end
	if type (text) ~= "string" or text == "" or string.find (text, "env-case-error", 1, true) then hard ("env_case (14, " .. index .. ") did not give a report: " .. first_line (text)) end
	keep ("sweep", text)
	parse_sweep (text)
	for _, row in ipairs (executed) do
		-- a compiler-compatibility case is reported apart from the safe-language cases: its output depends on the build's zero auto-init contract
		local scope = row.class == "compiler_zero_init" and "compiler_compatibility_case_canonical_build_flag" or "controlled_world_input"
		if row.class == "compiler_zero_init" then compiler_cases = compiler_cases + 1 end
		emit { event = "sweep_row", scope = scope, i = tonumber (row.i), class = row.class, rpm_w = row.rpm_w, out = row.out_raw, excepts = row.fp.excepts }
	end
	if probe then
		local calls, records = drain ("after the sweep", 1)
		sweep_calls = calls
		if #records ~= #executed then hard ("the probe recorded " .. #records .. " phase 1 calls where the sweep executed " .. #executed) end
		-- every actual call joins the chosen helicopter, case, player kind, flight model and declared input words, in order
		for k, record in ipairs (records) do
			local row, n, f = executed [k], record.n, record.f
			local what = "sweep call " .. (k - 1)
			if n.ent ~= sweep_entity.ent then hard (what .. " is for entity " .. n.ent .. " where the sweep chose " .. sweep_entity.ent) end
			if n.player ~= (fd_sweep and PLAYER_LOCAL or PLAYER_AI) or n.fd ~= (fd_sweep and 1 or 0) then hard (what .. " has player " .. n.player .. " and flight model " .. n.fd .. " where the " .. sweep .. " case declares otherwise") end
			if f.rpm_w ~= row.rpm_w then hard (what .. " recorded the rpm word " .. f.rpm_w .. " where the sweep declared " .. row.rpm_w) end
			for _, pair in ipairs ({ { "acc_x_w", "ax_w" }, { "acc_y_w", "ay_w" }, { "acc_z_w", "az_w" }, { "lrpm_w", "lrpm_w" }, { "rrpm_w", "rrpm_w" }, { "apu_w", "apu_w" } }) do
				if f [pair [1]] ~= row [pair [2]] then hard (what .. " recorded " .. pair [1] .. " " .. f [pair [1]] .. " where the sweep declared " .. row [pair [2]]) end
			end
			-- the probe's stored-after words of the sound children are the words the fixture read from the same children after the call (a join of two measurements, not an oracle)
			if #record.sounds ~= #row.out then hard (what .. " lists " .. #record.sounds .. " sound children where the fixture read " .. #row.out) end
			for position, sound in ipairs (record.sounds) do
				local expected = row.out [position]
				if tostring (sound.sub) ~= expected [1] or sound.after_amp ~= expected [2] or sound.after_pitch ~= expected [3] then
					hard (what .. " child " .. position .. " recorded sub " .. sound.sub .. " after " .. sound.after_amp .. ":" .. sound.after_pitch .. " where the fixture read " .. expected [1] .. " " .. expected [2] .. ":" .. expected [3])
				end
			end
		end
		emit { event = "rotor_frame", frame = frames + 1, calls = calls, scope = "controlled_sweep_" .. sweep }
		local map, line = report ()
		require_clean (map, line, "after the sweep")
		if map.calls_total ~= totals.calls or map.drained ~= totals.calls then hard ("the probe's calls (" .. map.calls_total .. " total, " .. map.drained .. " drained) are not the " .. totals.calls .. " lines read after the sweep: " .. line) end
		emit { event = "rotor_report", when = "after_sweep", raw = line }
	end
end

-- ---------------------------------------------------------------------------------------------------------------- what the driver JOINS (labelled derived; never a measurement)
emit {
	event = "derived", source = "the driver's join of the records above; not a measurement", probe = probe_arg,
	natural_calls = natural_calls, natural_phase0_calls = totals.phase0, controlled_phase1_calls = totals.phase1, calls = totals.calls, boot_calls = boot_calls, frames = frames,
	distinct_entities = distinct, calls_with_requests = totals.with_actions, calls_with_a_flight_model = totals.fd_calls, calls_where_the_seed_word_changed = totals.rng_changed, children_walked = totals.kids_seen,
	bytes = totals.bytes, sweep = sweep, sweep_executed = #executed, sweep_compiler_zero_init_cases = compiler_cases, last_seq = last_seq,
	coverage_note = "natural coverage is only the phase 0 calls; the phase 1 calls are controlled world input and are not natural coverage; compiler_zero_init sweep rows are compiler-compatibility cases under the build's declared zero auto-init contract, not safe-language oracles; no sound is claimed to play",
}
emit { event = "result", calls = totals.calls, natural_calls = natural_calls, sweep = sweep, probe = probe_arg }

-- the last thing the witness does is close its evidence, and a close that fails is a failure after the last row as much as before it
for _, name in ipairs (created) do
	local closed, close_error = files [name]:close ()
	if not closed then refuse ("OUTPUT FAILURE: closing " .. paths [name] .. " failed: " .. tostring (close_error)) end
end
io.stderr:write (string.format ("env-rotor-sounds: calls=%d natural=%d sweep=%s probe=%s\n", totals.calls, natural_calls, sweep, probe_arg))
