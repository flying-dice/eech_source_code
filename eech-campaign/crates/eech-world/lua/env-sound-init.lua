-- TEST-ONLY DRIVER of the ORIGINAL application sound initialisation and sound sample data characterization (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
--
-- WHAT IT ASKS. The ORIGINAL initialisation of the application sound system (appsound/snd_init.c initialise_application_sound_system), of the sound effect descriptors and the sound.hdr header
-- (appsound/snd_data.c initialise_application_sound_effects, open_application_sound_file_system), of the backend (modules/sound/soundman.c initialise_sound_system, through the headless build's null DirectSound
-- entry points) and the descriptor reads of the sound effect constructor (soundeff.c set_local_sound_effect_sample_indices), exactly as ONE boot of this process ran them. The facts come from the sound initialisation
-- probe (ss_envfx.h ENV_FXA; csrc/eech_env_observer.c), which records from process start and is armed by nothing, and are read by ONE read-only fixture case, group 14 index 15 (csrc/eech_env_fixture.c env_snd_init_case).
-- Nothing is created, nothing is drawn, no file is opened, no initialiser is called again and nothing needs restoring. This driver does not arm the observer, so the accepted event stream, the sound entity records and the old
-- cases are untouched. It moves nothing and decides nothing.
--
-- WHAT IT RECORDS (measured, by the original code): the command line flags at the moment of initialisation and which branch the original took (skipped by the no-sound or dedicated flag, or the backend initialisation
-- attempted and what it returned), the DirectSound entry points that were called and what they returned, the attempts to open sound.hdr and sound.dat and their results (zero attempts is a count of attempts, not a statement about the assets, and a descriptor size of 0 after the initialisation has several possible causes the rows can tell apart only in part: the header not opened, a descriptor with no entry in it, or a matched entry whose length is 0), the
-- header's sample count and how many descriptors found their entry in it (a matched entry of length 0 is a legitimate state), the descriptor catalogue's counts and a digest of every entry's (index, rate, size, original rate), the descriptor reads the constructor made
-- (calls, loops, samples, the distinct sample keys with their rate, size and original rate, a digest of every read), and whether the load routines ran. What it DERIVES from those rows is written in the `derived`
-- row and labelled as the driver's join of the observations, never as a measurement.
--
-- WHAT IT DOES NOT SHOW (printed by the fixture as `env-sndinit-unsupported`): a positive playing backend (the headless build has only the null DirectSound entry points; no device exists and no handle is created or faked),
-- a second initialisation, the original loader's data reads, and the other compiled descriptor readers (a static census is in the documentation, not measured here).
--
-- THIS DRIVER JUDGES NOTHING PHYSICAL. It checks the SHAPE and the COMPLETENESS of the capture (the rows in order, the keys in order, every value of its kind, the counts that must add up, no dropped or orphaned record) and
-- writes every row as it came. A report of another shape, an incomplete or partial capture, a missing probe, a refused case or a read-only probe that changed the shared seed word or the next free entity indices is a failure of
-- the WITNESS: a `witness_failure` row and an error beginning `env-sound-init:`. If the engine stops, that is `engine_failure` and the same error. Write, flush and close failures of the evidence are failures too.
--
-- THE RAW SIDECAR. The native's report of the case is the sole diagnostic capture of a 120 s run, so it is written, whole and exactly as it came, to a checked sidecar file (`raw_out`, default `<out>.case15.raw`) BEFORE the driver
-- validates a byte of it: a report the driver then refuses is still kept, and is never complete evidence (no `derived` or `result` row exists for it). It holds the native's private text and is not for Git.
--
-- EVIDENCE FILES ARE CREATED EXCLUSIVELY. Both `out` and the sidecar are made, before the engine is booted, by the host's `host.output_create_new` (src/output.rs): the operating system's atomic create-new open, which fails if anything of
-- that name exists (readable or not) and for a second spelling of a file this run has just created, so two names for one file, an existing output and an unreadable one are all refused before anything is booted and nothing is ever
-- truncated. There is NO fall back: Lua's `io.open` modes truncate, a read-open then write-open is not atomic, and the Windows Lua of this host does not create a file for the mode "wbx" (measured by Root, receipt
-- sound-init-exclusive-output-runtime-20261101), so a host without the primitive is refused. An engine that stops leaves an empty sidecar; every create, write, flush and close is checked.
--
-- THE REPORT'S SIZE. The case's report holds a row for each sample key the constructor read, up to the finite catalogue (the store row says how many entries it has); the buffer of this one case is REPORT_LIMIT bytes, derived from
-- that worst case in csrc/eech_env_observer.c, and a report at or over it is refused, never taken as complete.
--
-- host.args: root, out (JSON Lines path; created, it must not exist), raw_out (the raw sidecar of the native report; default <out>.case15.raw), scenario (lebanon_retail default | georgia_retail), seconds (0 to 600, default 0: natural frames run before the report so the constructor's reads are in it), frame_ms (10 to 1000, default 100)

io.stdout:setvbuf ("no")

local args = host.args
local function refuse (message) error ("env-sound-init: " .. message, 0) end
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
local seed = 1
local raw_path = args.raw_out == nil and (out_path .. ".case15.raw") or args.raw_out
if type (raw_path) ~= "string" or raw_path == "" or raw_path == out_path then refuse ("raw_out must be a path other than out") end
-- both evidence files are created exclusively, here, before anything is booted (see the header); the second spelling of a file the first just created fails like any other name that exists
local create = host.output_create_new
if type (create) ~= "function" then refuse ("this host has no host.output_create_new: evidence files cannot be created exclusively, and the driver does not fall back to a truncating open") end
local out, out_error = create (out_path)
if out == nil then refuse ("OUTPUT FAILURE: creating " .. out_path .. " failed: " .. tostring (out_error) .. " (evidence is created exclusively and never overwritten)") end
local raw, raw_error = create (raw_path)
if raw == nil then
	pcall (out.close, out)
	refuse ("OUTPUT FAILURE: creating " .. raw_path .. " failed: " .. tostring (raw_error) .. " (evidence is created exclusively and never overwritten; the same file under two names is refused)")
end

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
host.use_reference_arithmetic () -- the reference host's arithmetic profile, selected right after the installation is prepared (src/reference_arithmetic.rs)
-- the hooks exist only in a module built with the env-observer feature; this driver does NOT arm the observer (the sound initialisation probe is armed by nothing), it only needs the build
if type (dc.env_observer_arm) ~= "function" then
	emit { event = "refusal", reason = "env_observer_unavailable", detail = "the module was built without the env-observer feature" }
	refuse ("the module has no env_observer_arm function: build eech-dc with the env-observer feature")
end
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign,
	gunship = "apache", seed = seed, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = host.model, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, seed = seed, seconds = seconds, frame_ms = frame_ms, feature = "env-observer", group = 14, index = 15 }

local present, method = pcall (function () return engine.env_case end)
if not present or type (method) ~= "function" then
	emit { event = "refusal", reason = "env_case_unavailable", detail = "the module was built without the env-fixture feature" }
	refuse ("the engine has no env_case method: build eech-dc with the env-observer feature")
end

-- the natural frames the constructor's reads come from; the engine's own stop is the result of the frame that stopped it
local frames = math.floor (seconds * 1000 / frame_ms)
for n = 1, frames do
	local ok, err = pcall (engine.frame, engine, frame_ms)
	if not ok then
		emit { event = "engine_failure", frame = n, reason = string.match (tostring (err), "[^\r\n]+") or tostring (err) }
		refuse ("frame " .. n .. " FAILED in the engine: " .. (string.match (tostring (err), "[^\r\n]+") or tostring (err)))
	end
end

-- ---------------------------------------------------------------------------------------------------------------- the report grammar: key=value rows, kept as text
-- the buffer of this case's report: 320 KiB, the worst case of csrc/eech_env_observer.c (a row of at most 128 characters for each of the catalogue's keys, plus at most 16 KiB of fixed rows) with room to spare; Rust holds the same number
local REPORT_LIMIT = 320 * 1024
local function parse (text, what)
	if type (text) ~= "string" or text == "" or string.sub (text, -1) ~= "\n" then hard (what .. " is empty or truncated (it does not end with a newline)") end
	if #text >= REPORT_LIMIT then hard (what .. " does not fit the report buffer of " .. math.floor (REPORT_LIMIT / 1024) .. " KiB") end
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
local function is_digest (v) return type (v) == "string" and #v == 16 and string.find (v, "^[0-9a-f]+$") ~= nil end
local function is_count (v) return type (v) == "string" and #v <= 10 and string.find (v, "^%d+$") ~= nil end
local function is_flag (v) return v == "0" or v == "1" end
local function is_int (v) return type (v) == "string" and #v <= 11 and string.find (v, "^-?%d+$") ~= nil end
local function is_token (v) return type (v) == "string" and #v <= 200 and string.find (v, "^[%w_,]+$") ~= nil end
local KINDS = { w = is_word, d = is_digest, n = is_count, b = is_flag, i = is_int, s = is_token }
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

local what = "env_case(14, 15)"
local ok, text = pcall (engine.env_case, engine, 14, 15)
if not ok then
	pcall (raw.close, raw)
	emit { event = "engine_failure", i = 15, reason = string.match (tostring (text), "[^\r\n]+") or tostring (text) }
	refuse ("env_case(14, 15) FAILED in the engine: " .. (string.match (tostring (text), "[^\r\n]+") or tostring (text)))
end
-- the native's report is kept whole BEFORE any of it is validated
if type (text) ~= "string" then text = "" end
do
	local wrote, werr = raw:write (text)
	if not wrote then refuse ("OUTPUT FAILURE: writing " .. raw_path .. " failed: " .. tostring (werr)) end
	local flushed, ferr = raw:flush ()
	if not flushed then refuse ("OUTPUT FAILURE: flushing " .. raw_path .. " failed: " .. tostring (ferr)) end
	local closed_raw, cerr = raw:close ()
	if not closed_raw then refuse ("OUTPUT FAILURE: closing " .. raw_path .. " failed: " .. tostring (cerr)) end
	emit { event = "raw_report", bytes = #text, sidecar = raw_path, note = "the native report exactly as returned, kept before validation; not complete evidence by itself" }
end
if string.find (text, "env-case-error", 1, true) then hard (what .. " refused: " .. text) end
local rows = parse (text, what)

-- the rows in the order the C prints them; `at` walks them
local at = 1
local function take (event, spec)
	local row = rows [at]
	if row == nil or row.event ~= event then hard (what .. " does not give the row " .. event .. " where it belongs (row " .. at .. "): " .. (row and row.raw or "end of the report")) end
	check (row, spec, what)
	at = at + 1
	return row
end
local function number (row, key) return tonumber (row.map [key]) end

local CAPACITY = 4
local MAX_ATTEMPTS, MAX_SIDES = 8, 8
local plan = take ("env-sndinit-plan", "version=1 read_only=1 process_isolated=not_needed comms:i")
local store = take ("env-sndinit-store", "version=1 boots:n stored:n capacity=4 dropped:n orphans:n opens_outside_effects:n catalogue_entries:n saturated:n")
local boots, stored = number (store, "boots"), number (store, "stored")
-- completeness of the capture: an initialisation that was never recorded (a native without the hooks cannot answer this case, but a boot that ran none of them could), a table that overflowed, a point that had no
-- initialisation to belong to
if boots < 1 then hard ("the probe recorded no initialisation of the application sound system: " .. store.raw) end
if stored ~= math.min (boots, CAPACITY) then hard ("the probe stored " .. stored .. " of " .. boots .. " initialisations (capacity " .. CAPACITY .. "): " .. store.raw) end
if number (store, "dropped") ~= 0 then hard ("the probe dropped " .. store.map.dropped .. " records: the capture is incomplete: " .. store.raw) end
-- a counter of the probe that reached its maximum stopped counting (it never wraps) and raised the latch: whatever the other rows say, the capture is not complete
if number (store, "saturated") ~= 0 then hard ("a counter of the probe reached its limit and stopped counting (" .. store.map.saturated .. " times): the capture is incomplete: " .. store.raw) end
if number (store, "orphans") ~= 0 then hard ("the probe saw " .. store.map.orphans .. " points that belong to no initialisation: the capture is incomplete: " .. store.raw) end
if boots > CAPACITY then hard ("the process initialised the sound system " .. boots .. " times and the probe keeps " .. CAPACITY .. ": the capture is incomplete: " .. store.raw) end
if number (store, "catalogue_entries") < 1 then hard ("the descriptor catalogue has no entries: " .. store.raw) end

local boot_rows, catalogues = {}, {}
for n = 0, stored - 1 do
	local boot = take ("env-sndinit-boot", "n:n no_sound:i dedicated:i enabled_before:i initialised_before:i effects_begin:n filled:n effects_end:n open_true:n open_false:n header_count:i matched:n missed:n " ..
		"backend:n enabled_backend:i initialised_backend:i load:n skipped:n exited:n enabled_exit:i")
	if number (boot, "n") ~= n then hard ("the initialisation rows are out of order (expected " .. n .. "): " .. boot.raw) end
	local filled = take ("env-sndinit-catalogue", "n:s when=filled named:n nonzero_size:n nonzero_rate:n nonzero_orate:n digest_w:d")
	local finished = take ("env-sndinit-catalogue", "n:s when=end named:n nonzero_size:n nonzero_rate:n nonzero_orate:n digest_w:d")
	if filled.map.n ~= tostring (n) or finished.map.n ~= tostring (n) then hard ("the catalogue rows of initialisation " .. n .. " carry another ordinal: " .. filled.raw .. " | " .. finished.raw) end
	-- a record of an initialisation that did not run to its end is a partial capture, not a result
	for _, key in ipairs ({ "effects_begin", "filled", "effects_end", "exited" }) do
		if number (boot, key) ~= 1 then hard ("initialisation " .. n .. " has " .. key .. "=" .. boot.map [key] .. ": its capture is partial: " .. boot.raw) end
	end
	if number (boot, "open_true") + number (boot, "open_false") ~= 1 then hard ("initialisation " .. n .. " recorded " .. (number (boot, "open_true") + number (boot, "open_false")) .. " results of the header open, not one: " .. boot.raw) end
	if number (boot, "backend") + number (boot, "skipped") ~= 1 then hard ("initialisation " .. n .. " took " .. (number (boot, "backend") + number (boot, "skipped")) .. " branches of the backend gate, not one: " .. boot.raw) end
	if number (boot, "load") > 1 then hard ("initialisation " .. n .. " ran the sample loader " .. boot.map.load .. " times: " .. boot.raw) end
	boot_rows [n] = boot
	catalogues [n] = { filled = filled, finished = finished }
end
local now = take ("env-sndinit-catalogue", "n=now when=report named:n nonzero_size:n nonzero_rate:n nonzero_orate:n digest_w:d")
local flags_now = take ("env-sndinit-now", "no_sound:i dedicated:i enabled:i initialised:i blocked_count:i application_samples:i")
local samples = take ("env-sndinit-samples", "lightning_rate:i lightning_size:i lightning_orate:i thunder_rate:i thunder_size:i thunder_orate:i")
local sides = take ("env-sndinit-load-side", "calls:n recorded:n")
if number (sides, "recorded") ~= math.min (number (sides, "calls"), MAX_SIDES) then hard ("the load-side calls recorded " .. sides.map.recorded .. " of " .. sides.map.calls .. ": " .. sides.raw) end
-- a table that cannot hold every call is a PARTIAL capture, not a shorter complete one: the probe keeps MAX_SIDES calls and says how many there were
if number (sides, "calls") > MAX_SIDES then hard ("the probe recorded only " .. sides.map.recorded .. " of " .. sides.map.calls .. " load-side calls (it keeps " .. MAX_SIDES .. "): the capture is incomplete: " .. sides.raw) end
local side_rows = {}
for n = 0, number (sides, "recorded") - 1 do
	local call = take ("env-sndinit-load-side-call", "n:n side:i enabled:i no_sound:i dedicated:i")
	if number (call, "n") ~= n then hard ("the load-side call rows are out of order: " .. call.raw) end
	side_rows [#side_rows + 1] = call
end
local header = take ("env-sndinit-file", "kind=header attempts:n ok:n")
local data = take ("env-sndinit-file", "kind=data attempts:n ok:n")
if number (header, "ok") > number (header, "attempts") or number (data, "ok") > number (data, "attempts") then hard ("a file has more successful opens than attempts: " .. header.raw .. " | " .. data.raw) end
local file_log = take ("env-sndinit-file-log", "attempts:n recorded:n")
if number (file_log, "attempts") ~= number (header, "attempts") + number (data, "attempts") then hard ("the file log counts " .. file_log.map.attempts .. " attempts but the two files " .. (number (header, "attempts") + number (data, "attempts")) .. ": " .. file_log.raw) end
if number (file_log, "recorded") ~= math.min (number (file_log, "attempts"), MAX_ATTEMPTS) then hard ("the file log recorded " .. file_log.map.recorded .. " of " .. file_log.map.attempts .. ": " .. file_log.raw) end
if number (file_log, "attempts") > MAX_ATTEMPTS then hard ("the probe recorded only " .. file_log.map.recorded .. " of " .. file_log.map.attempts .. " file attempts (it keeps " .. MAX_ATTEMPTS .. "): the capture is incomplete: " .. file_log.raw) end
local attempt_rows = {}
for n = 0, number (file_log, "recorded") - 1 do
	local attempt = take ("env-sndinit-file-attempt", "n:n kind:s ok:b installation_root_retry:b read_mode:b")
	if number (attempt, "n") ~= n or (attempt.map.kind ~= "header" and attempt.map.kind ~= "data") then hard ("the file attempt rows are out of order or of another kind: " .. attempt.raw) end
	attempt_rows [#attempt_rows + 1] = attempt
end
-- the counters of each file are the kept attempts, all of them (the log is complete here): the attempts of its kind and the ones that succeeded
for _, file in ipairs ({ { "header", header }, { "data", data } }) do
	local kept, kept_ok = 0, 0
	for _, attempt in ipairs (attempt_rows) do
		if attempt.map.kind == file [1] then
			kept = kept + 1
			if attempt.map.ok == "1" then kept_ok = kept_ok + 1 end
		end
	end
	if kept ~= number (file [2], "attempts") or kept_ok ~= number (file [2], "ok") then
		hard ("the kept attempts of the " .. file [1] .. " file (" .. kept .. " attempts, " .. kept_ok .. " ok) do not reconcile with its counters (attempts " .. file [2].map.attempts .. ", ok " .. file [2].map.ok .. "): " .. file [2].raw)
	end
end
local COM_NAMES = { "DirectSoundCreate", "IDirectSound_SetCooperativeLevel", "IDirectSound_CreateSoundBuffer", "IDirectSound_Release" }
local com_rows = {}
for _, name in ipairs (COM_NAMES) do
	local com = take ("env-sndinit-com", "function:s calls:n last_return_w:w")
	if com.map ["function"] ~= name then hard ("the DirectSound rows are out of order (expected " .. name .. "): " .. com.raw) end
	com_rows [name] = com
end
local ctor = take ("env-sndinit-ctor", "calls:n looping:n max_count:n sample_reads:n distinct:n out_of_domain:n digest_w:d")
if number (ctor, "looping") > number (ctor, "calls") then hard ("more looping constructor calls than calls: " .. ctor.raw) end
-- the kept keys are the keys of the catalogue (a table indexed by the key): there cannot be more of them than it has entries
if number (ctor, "distinct") > number (store, "catalogue_entries") then hard ("more distinct samples (" .. ctor.map.distinct .. ") than the catalogue has entries (" .. store.map.catalogue_entries .. "): " .. ctor.raw) end
local sample_rows, uses, seen_keys = {}, 0, {}
for n = 0, number (ctor, "distinct") - 1 do
	local sample = take ("env-sndinit-sample", "n:n index:i rate:i size:i orate:i uses:n")
	if number (sample, "n") ~= n then hard ("the sample rows are out of order: " .. sample.raw) end
	-- a kept sample is a sample key inside the reported catalogue, kept once, and was read at least once (that is why it is kept)
	if number (sample, "index") < 0 or number (sample, "index") >= number (store, "catalogue_entries") then hard ("the sample key " .. sample.map.index .. " is outside the catalogue (0 to " .. (number (store, "catalogue_entries") - 1) .. "): " .. sample.raw) end
	-- the key is its NUMBER, as the range test above read it: "2" and "02" (and "0" and "-0") are one key, so the table is keyed by the parsed value, never by the spelling
	local key = string.format ("%d", number (sample, "index"))
	if seen_keys [key] then hard ("the sample key " .. key .. " appears twice in the kept samples (" .. sample.map.index .. " repeats the key of an earlier row): " .. sample.raw) end
	seen_keys [key] = true
	if number (sample, "uses") < 1 then hard ("kept sample row " .. n .. " has no use: " .. sample.raw) end
	uses = uses + number (sample, "uses")
	sample_rows [#sample_rows + 1] = sample
end
-- every descriptor read is either a use of a kept sample or a read whose key is outside the catalogue (counted, and refused below): a table that does not add up is not a capture
if uses + number (ctor, "out_of_domain") ~= number (ctor, "sample_reads") then hard ("the constructor's sample reads (" .. ctor.map.sample_reads .. ") are not the uses of the kept samples (" .. uses .. ") plus the out-of-domain reads (" .. ctor.map.out_of_domain .. "): " .. ctor.raw) end
if number (ctor, "out_of_domain") ~= 0 then hard ("the constructor read " .. ctor.map.out_of_domain .. " samples whose key is outside the catalogue (0 to " .. (number (store, "catalogue_entries") - 1) .. "): the capture is incomplete: " .. ctor.raw) end
local unsupported = {}
for _ = 1, 4 do unsupported [#unsupported + 1] = take ("env-sndinit-unsupported", "item:s reason:s") end
local fingerprint = take ("env-sndinit-fingerprint", "rng_before_w:w rng_after_w:w entity_before:i entity_after:i local_before:i local_after:i unchanged:b")
if fingerprint.map.unchanged ~= "1" or fingerprint.map.rng_before_w ~= fingerprint.map.rng_after_w or fingerprint.map.entity_before ~= fingerprint.map.entity_after or fingerprint.map.local_before ~= fingerprint.map.local_after then
	hard ("the read-only probe changed the shared seed word or the next free entity indices: " .. fingerprint.raw)
end
local finish = take ("env-sndinit-end", "changed=0 read_only=1")
if rows [at] ~= nil then hard (what .. " has a row after its end row: " .. rows [at].raw) end

-- every row, as it came, with its values kept as text
for _, row in ipairs (rows) do
	local t = { event = row.event, scope = "case", raw = row.raw }
	for _, k in ipairs (row.keys) do t [k] = row.map [k] end
	emit (t)
end

-- ---------------------------------------------------------------------------------------------------------------- what the driver JOINS (labelled derived; never a measurement)
local first = boot_rows [0]
local branch
if number (first, "skipped") == 1 then
	branch = "skipped_by_flags"
elseif number (first, "enabled_backend") ~= 0 then
	branch = "backend_enabled"
else
	branch = "backend_initialisation_returned_false"
end
local function reached ()
	local last = "none"
	for _, name in ipairs (COM_NAMES) do
		if number (com_rows [name], "calls") > 0 then last = name end
	end
	return last
end
local unchanged_since = (boots == 1) and (now.map.digest_w == catalogues [0].finished.map.digest_w)
emit {
	event = "derived", source = "the driver's join of the rows above; not a measurement",
	boots = boots, first_branch = branch, flags_agree_with_branch = ((first.map.no_sound ~= "0" or first.map.dedicated ~= "0") == (number (first, "skipped") == 1)),
	header_opened_in_effects = number (first, "open_true") == 1, header_open_attempts = number (header, "attempts"), header_opens_ok = number (header, "ok"),
	data_open_attempts = number (data, "attempts"), data_opens_ok = number (data, "ok"),
	directsound_last_entry_called = reached (), descriptors_matched = number (first, "matched"), descriptors_missed = number (first, "missed"),
	catalogue_unchanged_since_initialisation = unchanged_since, constructor_calls = number (ctor, "calls"), constructor_distinct_samples = number (ctor, "distinct"),
	load_routines_ran = number (first, "load") == 1, load_side_calls = number (sides, "calls"),
}
emit { event = "result", boots = boots, branch = branch, frames = frames, comms = plan.map.comms, changed = finish.map.changed, read_only = finish.map.read_only }

-- the last thing the witness does is close its evidence, and a close that fails is a failure after the last row as much as before it
local closed, close_error = out:close ()
if not closed then refuse ("OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error)) end
io.stderr:write (string.format ("env-sound-init: boots=%d branch=%s\n", boots, branch))
