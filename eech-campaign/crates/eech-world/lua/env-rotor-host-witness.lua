-- TEST-ONLY PUBLIC-HOST OWNERSHIP WITNESS of the helicopter rotor acoustics (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. A script only: no C, no ABI, no provider and no host solver is written here.
--
-- THE QUESTION. The original `update_local_helicopter_rotor_sounds` computed every rotor, slap, turbine and APU level and pitch, and chose the pause and resume requests, itself. That arithmetic and that choice are
-- now the HOST's: the campaign reaches the host through the public path (host-owned Lua -> `eech_dc` -> the campaign -> the host's provider callback) as the physical operation `rotor_sound`, and the host answers the
-- levels and pitches of the logical channels and the ordered requests (docs/d3-sound-lifetime.md and eech-world/src/env.rs have the request and the reply). This witness asks, on the native it is run against, whether
-- the campaign really asks the host once for every call of the function and whether the host's ANSWER is what EVERY real sound child and EVERY real pause or resume request shows.
--
-- WHAT IT DOES. It creates its own record EXCLUSIVELY (`host.output_create_new`) BEFORE anything else runs, wraps the `host.env_provider` factory BEFORE the delegate boots and counts ONLY the requests whose `op` is
-- `rotor_sound`; every other request is forwarded UNCHANGED to the original provider. For each `rotor_sound` request it retains, as numbers, the request the campaign SENT, the answer the reference host gave (the
-- REFERENCE) and the answer it returned to the campaign (the RETURNED; they differ only in the modes that alter the answer). It then runs the EXISTING original-rotor driver `lua/env-rotor-sounds.lua` (default
-- `sweep=ai`, `seconds=0`) through `dofile`, so the delegate's argument checks, strict record grammar, raw sidecars and exclusive-output handling are exactly its own, with no second parser of its rules, and judges the
-- delegate's own complete artifacts: the Nth request is JOINED to the Nth recorded call (`<out>.rotor.raw`), and then
--   * the recorded inputs of the call equal the SENT request (role, flight model, rpm, acceleration, engine words), so a request that does not carry what the function read is caught;
--   * EVERY sound child of the call, duplicates included, shows the RETURNED answer: the amplification word of its channel and, except for the winding channel, the pitch word; the winding pitch and a child of a
--     sub-type the original does not handle are unchanged from their recorded `before` words (no-write evidence only where the probe measured both words);
--   * the recorded pause and resume requests are the RETURNED actions: count, order, kind, sub-type and delay word;
--   * every row of `<out>.sweep.raw` is joined to its phase-1 call and shows the same words.
-- No aggregate count stands for this evidence. The words are compared; no level, pitch or action is computed here, and nothing is a formula oracle.
--
-- MODES (`mode=`, default healthy):
--   healthy    the request is answered by the reference host unchanged. REQUIRED: the delegate completed, its artifacts are complete and readable, the host was asked EXACTLY once for every call the probe recorded and
--              every join above holds. Against the pre-extraction native (for instance the original measured native e54423b5c) the campaign computed the levels itself and the host was asked 0 times: the error
--              `rotor-host-capability-missing` (the RED of the capability, not a physical regression).
--   changed    the reference host answers, and the witness then CHANGES the answer in a valid way: the looping channel's level and pitch, the slap channel's level and the delay of every pause. REQUIRED: everything healthy
--              requires, and for EVERY request the returned answer carries exactly those changes and nothing else (every other channel and action equals the REFERENCE), so every child and request that shows the returned
--              answer shows the changed targets and the reference words for everything unaffected. Every sweep call needs a looping and a slap child and the sweep a pause; the answers must really differ from the reference.
--   withheld   the host raises an error. REQUIRED: the delegate FAILS at `[env-provider:rotor_sound.callback]` carrying this witness's own withdrawal, after exactly one host request, with no result row.
--   malformed  the reference answer plus a pitch on the winding channel, which writes an amplification only. REQUIRED: the delegate fails at `[env-provider:rotor_sound.answer]` naming the winding channel, after exactly one
--              host request, with no result row.
-- A STOP is judged from the delegate's own readable artifacts, never assumed empty: the first request is made when the campaign first runs the function, which is inside `dc.boot` (no row yet written: every artifact
-- verified empty) or in a later frame or fixture case (the delegate's `engine_failure` row is the last row of its record and names the marker). A stop is the expected evidence of withheld and malformed, not a
-- completed success and not a fallback. A campaign that completes anyway, or stops at another marker or for another cause, is an error.
--
-- EVIDENCE I/O IS CHECKED. Every artifact is read with a checked open, read and close; a missing, unreadable or unclosable artifact is a SETUP FAILURE and never proof that there is no result (an absent result is
-- established only from a readable record). The witness's own record is created exclusively before the delegate and written, flushed and closed with every result checked; a path equal to, an alias of or an existing
-- file at any of `out`, its three sidecars and the record is refused (before anything runs, or by the delegate's own exclusive creation) without destroying anything, and a failed capture is left as found.
--
-- VERDICTS (errors begin `env-rotor-host-witness:`): `SETUP FAILURE` -- the delegate failed (healthy, changed), an artifact is unreadable or incomplete, or the evidence cannot be joined: NOT a verdict on ownership;
-- `rotor-host-capability-missing` -- the delegate completed and the host was asked 0 times; `ownership-not-shown` -- the host was asked but the real record does not show its answer (a wrong number of requests, or
-- a child, a request or a row that does not show the returned answer); `not-stopped` / `wrong-boundary` -- a withheld or malformed answer did not stop the campaign at its marker.
--
-- WHAT IT DOES NOT. It asserts no private type, struct layout or ABI, picks no physical outcome beyond the reference host's own, exercises no renderer, speaker, sound backend or network, and proves no audibility. It
-- retains every request, so `seconds` is limited to 10. It says nothing of the other physical responsibilities of D3.
--
-- ARGUMENTS (host.args, as the host executable passes them; the witness is the SCRIPT the host runs and the delegate is loaded from a path):
--   eech-world <this script> root=<installation root> out=<delegate JSON Lines path> delegate=<ABSOLUTE path of lua/env-rotor-sounds.lua> [mode=healthy|changed|withheld|malformed] [sweep=ai|fd (default ai)]
--                                                                                  [scenario=lebanon_retail|georgia_retail] [seconds=<0..10>] [authority_out=<this witness's record path; default <out>.authority.jsonl>]
--   `root`, `out`, `scenario`, `seconds` and `sweep` are the delegate's own arguments and are passed through unchanged (the delegate reads the same `host.args`; `probe=off` is refused here because the probe's record is the
--   evidence). One fresh process and root per run.

io.stdout:setvbuf ("no")

local NAME = "env-rotor-host-witness"
local args = host.args
local function refuse (message) error (NAME .. ": " .. message, 0) end

local delegate_path = args.delegate
if type (delegate_path) ~= "string" or delegate_path == "" then refuse ("delegate=<absolute path of env-rotor-sounds.lua> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<the delegate's JSON Lines path> is required") end
local mode = args.mode == nil and "healthy" or args.mode
local MARKERS = { withheld = "[env-provider:rotor_sound.callback]", malformed = "[env-provider:rotor_sound.answer]" }
if mode ~= "healthy" and mode ~= "changed" and mode ~= "withheld" and mode ~= "malformed" then
	refuse ("mode must be healthy, changed, withheld or malformed, got '" .. tostring (mode) .. "'")
end
local MARKER = MARKERS [mode]
if args.sweep == nil then args.sweep = "ai" end
if args.sweep ~= "ai" and args.sweep ~= "fd" then refuse ("sweep must be ai or fd, got '" .. tostring (args.sweep) .. "'") end
if args.probe ~= nil and args.probe ~= "on" then refuse ("probe must be on: the probe's record is the evidence, got '" .. tostring (args.probe) .. "'") end
local seconds = args.seconds == nil and 0 or tonumber (args.seconds)
if seconds == nil or seconds ~= math.floor (seconds) or seconds < 0 or seconds > 10 then refuse ("seconds must be an integer from 0 to 10 (the witness retains every request), got '" .. tostring (args.seconds) .. "'") end
local authority_path = args.authority_out
if authority_path == nil then authority_path = out_path .. ".authority.jsonl" end
if type (authority_path) ~= "string" or authority_path == "" then refuse ("authority_out must be a path") end

-- the delegate's artifacts, and the record: five different names, each created exclusively (the second spelling of a file this run created, or any existing file, makes the creation fail)
local ARTIFACTS = { { "out", out_path }, { "rotor", out_path .. ".rotor.raw" }, { "sndinit", out_path .. ".sndinit.raw" }, { "sweep", out_path .. ".sweep.raw" } }
for _, artifact in ipairs (ARTIFACTS) do
	if artifact [2] == authority_path then refuse ("authority_out must not be the delegate's " .. artifact [1] .. " artifact " .. artifact [2]) end
end
local create = host.output_create_new
if type (create) ~= "function" then refuse ("this host has no host.output_create_new: the record cannot be created exclusively, and the witness does not fall back to a truncating open") end
local authority, authority_why = create (authority_path)
if not authority then refuse ("OUTPUT FAILURE: creating " .. authority_path .. " failed: " .. tostring (authority_why) .. " (the record is created exclusively and never overwritten; nothing was run)") end

-- ---------------------------------------------------------------------------------------------------------------- the record (every write, flush and close checked; written once)
local requests, forwarded, withdrawals, altered, base_errors = 0, 0, 0, 0, 0
local ran, problem = false, nil
local recorded = false
local function record (completed, verdict, extra)
	if recorded then return end
	recorded = true
	local line = string.format ('{"altered":%d,"base_errors":%d,"completed":%s,"delegate_ran":%s,"forwarded_other_requests":%d,"mode":"%s","op":"rotor_sound","requests":%d,%s"source":"host.env_provider wrapper on the public path","sweep":"%s","verdict":"%s","withdrawals":%d}\n',
		altered, base_errors, completed and "true" or "false", ran and "true" or "false", forwarded, mode, requests, extra or "", args.sweep, verdict, withdrawals)
	local ok, err = authority:write (line)
	if not ok then refuse ("OUTPUT FAILURE: writing " .. authority_path .. " failed: " .. tostring (err)) end
	ok, err = authority:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. authority_path .. " failed: " .. tostring (err)) end
	ok, err = authority:close ()
	if not ok then refuse ("OUTPUT FAILURE: closing " .. authority_path .. " failed: " .. tostring (err)) end
end
local function fail (verdict, message, completed, extra)
	record (completed or false, verdict, extra)
	refuse (message)
end

-- ---------------------------------------------------------------------------------------------------------------- words: an exact single-precision number is compared as the IEEE word the probe prints
local function hex8 (bits) return string.format ("%04x%04x", math.floor (bits / 65536), bits % 65536) end
local ZERO = "00000000"
local function word_of (x)
	if type (x) ~= "number" or x ~= x then return nil end
	local negative = x < 0 or (x == 0 and 1 / x < 0)
	if negative then x = -x end
	local bits
	if x == math.huge then
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
			mantissa = x * 2 ^ 149
		end
		if mantissa ~= math.floor (mantissa) or mantissa < 0 or mantissa >= 8388608 then return nil end
		bits = exponent * 8388608 + mantissa
	end
	if negative then bits = bits + 2147483648 end
	return hex8 (bits)
end

local function deep_copy (value)
	if type (value) ~= "table" then return value end
	local copy = {}
	for key, inner in pairs (value) do copy [key] = deep_copy (inner) end
	return copy
end

-- the words of the changed answer (all exact single-precision values)
local CHANGED = {
	looping_amplification = 0.375, looping_pitch = 0.8125, slap_amplification = 0.1875, delay = 2.0,
}

-- ---------------------------------------------------------------------------------------------------------------- the wrapper: forward everything, count only the op, retain what was sent and answered
local log = {}
local WITHDRAWAL = NAME .. ": the intentional withdrawal of the rotor_sound answer"
local factory = host.env_provider
if type (factory) ~= "function" then fail ("setup_failure_no_provider_factory", "this host has no host.env_provider factory") end

local wrapped = function (...)
	local base = factory (...)
	if type (base) ~= "function" then error (NAME .. ": host.env_provider () did not return a provider function", 0) end
	return function (request, ...)
		if type (request) == "table" and request.op == "rotor_sound" then
			requests = requests + 1
			-- the withdrawal is this witness's own, made before the reference provider is consulted, so an error of that provider cannot be taken for it
			if mode == "withheld" then
				withdrawals = withdrawals + 1
				error (WITHDRAWAL, 0)
			end
			local consulted, answer = pcall (base, request, ...)
			if not consulted then
				base_errors = base_errors + 1
				error (answer, 0)
			end
			local entry = { sent = deep_copy (request), reference = deep_copy (answer) }
			if mode == "malformed" then
				if type (answer) == "table" and type (answer.channels) == "table" and type (answer.channels.winding) == "table" then
					answer.channels.winding.pitch = 1.0
					altered = altered + 1
				end
			elseif mode == "changed" and type (answer) == "table" and type (answer.channels) == "table" and type (answer.channels.looping) == "table" and type (answer.channels.slap) == "table" then
				answer.channels.looping.amplification = CHANGED.looping_amplification
				answer.channels.looping.pitch = CHANGED.looping_pitch
				answer.channels.slap.amplification = CHANGED.slap_amplification
				for _, action in ipairs (answer.actions or {}) do
					if action.action == "pause" then action.delay = CHANGED.delay end
				end
				altered = altered + 1
			end
			entry.returned = deep_copy (answer)
			log [#log + 1] = entry
			return answer
		end
		forwarded = forwarded + 1
		return base (request, ...)
	end
end
local installed, install_error = pcall (function () host.env_provider = wrapped end)
if not installed or host.env_provider ~= wrapped then fail ("setup_failure_provider_not_wrapped", "the host table does not accept a wrapped env_provider: " .. tostring (install_error)) end

-- ---------------------------------------------------------------------------------------------------------------- the delegate, exactly as it is
ran, problem = pcall (dofile, delegate_path)
host.env_provider = factory

-- a CHECKED read of an evidence file: its text, or nil and the reason (a failed open, read or close is never an empty file and never "no result")
local function read_checked (path)
	local file, open_error = io.open (path, "r")
	if not file then return nil, "open failed: " .. tostring (open_error) end
	local text, read_error = file:read ("*a")
	if text == nil then
		pcall (file.close, file)
		return nil, "read failed: " .. tostring (read_error)
	end
	local closed, close_error = file:close ()
	if not closed then return nil, "close failed: " .. tostring (close_error) end
	return text
end
local function has (text, fragment) return text ~= nil and string.find (text, fragment, 1, true) ~= nil end

local delegate_error = ran and "" or tostring (problem)
-- the delegate's own exclusive creation of an artifact failed (an existing file, or a second name for the record or another artifact): nothing was run and nothing was destroyed
if not ran and has (delegate_error, "OUTPUT FAILURE") then
	fail ("setup_failure_delegate_could_not_write_evidence", "SETUP FAILURE: the delegate could not create or write its evidence (an existing file, or the record " .. authority_path .. " is the same file as one of its artifacts): " .. delegate_error)
end

-- healthy and changed need a delegate that completed: its failure is reported as such before any artifact is judged
if not MARKER and not ran then
	fail ("setup_failure_delegate_failed", "SETUP FAILURE: the delegate failed: " .. delegate_error)
end

-- every artifact of the delegate, read with checked I/O; unreadable evidence is never "nothing happened"
local texts = {}
for _, artifact in ipairs (ARTIFACTS) do
	local text, why = read_checked (artifact [2])
	if text == nil then
		fail ("setup_failure_evidence_unreadable", "SETUP FAILURE: the delegate's " .. artifact [1] .. " artifact " .. artifact [2] .. " could not be read (" .. why .. "): there is no verdict without it")
	end
	texts [artifact [1]] = text
end
local function rows_with (text, event)
	local found = {}
	for line in string.gmatch (text, "[^\n]+") do
		if has (line, '"event":"' .. event .. '"') then found [#found + 1] = line end
	end
	return found
end
local result_rows = rows_with (texts.out, "result")

-- ---------------------------------------------------------------------------------------------------------------- withheld and malformed: the campaign must STOP at the named boundary
if MARKER then
	if ran then
		record (#result_rows > 0, "not_stopped")
		refuse ("not-stopped: the host answered its 'rotor_sound' request wrongly (" .. mode .. ", " .. requests .. " request(s)) and the delegate completed: the campaign did not stop at " .. MARKER)
	end
	if requests == 0 then
		fail ("setup_failure_the_host_was_never_asked", "SETUP FAILURE: the delegate stopped but the host was asked 0 times for 'rotor_sound': the stop is not the " .. mode .. " answer's: " .. delegate_error)
	end
	if base_errors > 0 then
		fail ("setup_failure_base_provider_error", "SETUP FAILURE: the reference provider raised an error " .. base_errors .. " time(s) before the " .. mode .. " refusal could be made: the stop is the provider's, not the intended refusal's: " .. delegate_error)
	end
	if requests ~= 1 or (mode == "withheld" and withdrawals ~= 1) or (mode == "malformed" and altered ~= 1) then
		fail ("setup_failure_no_single_refusal", "SETUP FAILURE: the host made " .. requests .. " request(s), " .. withdrawals .. " withdrawal(s) and " .. altered .. " altered answer(s) where the refusal is one request refused once: " .. delegate_error)
	end
	if #result_rows > 0 then
		fail ("contradictory_result_row", "contradictory-result: the delegate wrote a result row although the campaign is said to have stopped: " .. delegate_error)
	end
	if not has (delegate_error, MARKER) then
		fail ("wrong_boundary", "wrong-boundary: the delegate stopped, but not at " .. MARKER .. ": " .. delegate_error)
	end
	if mode == "withheld" and not has (delegate_error, WITHDRAWAL) then
		fail ("setup_failure_withdrawal_not_the_cause", "SETUP FAILURE: the campaign stopped at " .. MARKER .. " but the stop does not carry this witness's withdrawal (" .. WITHDRAWAL .. "): the callback failed for another reason: " .. delegate_error)
	end
	if mode == "malformed" and not has (delegate_error, "winding") then
		fail ("wrong_reason", "wrong-reason: the campaign stopped at " .. MARKER .. " but the refusal does not name the winding channel that was given a pitch: " .. delegate_error)
	end
	-- where it stopped is read from the delegate's record, not assumed: before any row (inside boot, every artifact verified empty) or at the engine_failure row that ends a record
	local branch
	if texts.out == "" then
		for _, artifact in ipairs (ARTIFACTS) do
			if texts [artifact [1]] ~= "" then
				fail ("unexpected_evidence_state", "unexpected-evidence: the campaign stopped before the delegate wrote its first row but its " .. artifact [1] .. " artifact holds " .. #texts [artifact [1]] .. " bytes")
			end
		end
		branch = "boot"
	else
		for _, event in ipairs ({ "witness_failure", "refusal" }) do
			if #rows_with (texts.out, event) > 0 then
				fail ("setup_failure_delegate_refused", "SETUP FAILURE: the delegate's record holds a " .. event .. " row, so the stop is the delegate's own and not the engine's: " .. delegate_error)
			end
		end
		local failures = rows_with (texts.out, "engine_failure")
		local last
		for line in string.gmatch (texts.out, "[^\n]+") do last = line end
		if #failures ~= 1 or last ~= failures [1] then
			fail ("setup_failure_no_engine_failure_row", "SETUP FAILURE: the delegate's record does not end in exactly one engine_failure row, so the failure is not the engine's stop: " .. delegate_error)
		end
		if not has (failures [1], MARKER) or (mode == "withheld" and not has (failures [1], WITHDRAWAL)) or (mode == "malformed" and not has (failures [1], "winding")) then
			fail ("wrong_boundary", "wrong-boundary: the delegate's engine_failure row does not carry the " .. mode .. " refusal at " .. MARKER .. ": " .. failures [1])
		end
		branch = "engine_failure_row"
	end
	record (false, "stopped_at_boundary", string.format ('"branch":"%s","marker":"%s","stopped":true,', branch, MARKER))
	io.stderr:write (string.format ("%s: mode=%s stopped at %s (%s) after %d host request(s) for 'rotor_sound'\n", NAME, mode, MARKER, branch, requests))
	return
end

-- ---------------------------------------------------------------------------------------------------------------- healthy and changed: the delegate must have completed and EVERY child and request must show the host's answer
if #result_rows ~= 1 then
	fail ("setup_failure_no_result_row", "SETUP FAILURE: the delegate returned but its record holds " .. #result_rows .. " result rows where one is required: the run is incomplete")
end
for _, event in ipairs ({ "witness_failure", "refusal", "engine_failure" }) do
	if #rows_with (texts.out, event) > 0 then
		fail ("setup_failure_delegate_refused", "SETUP FAILURE: the delegate returned but its record holds a " .. event .. " row")
	end
end

-- the probe's records, as the delegate has already validated them (its grammar is closed): the fields this witness joins
local function fields_of (line)
	local map = {}
	for key, value in string.gmatch (line, "([%w_]+)=(%S+)") do map [key] = value end
	return map
end
local function list_of (text, pattern, what)
	local items = {}
	if text == "-" then return items end
	for item in string.gmatch (text, "[^,]+") do
		local parts = { string.match (item, pattern) }
		if #parts == 0 then error ({ evidence = what .. " item '" .. item .. "' is not in the delegate's grammar" }) end
		items [#items + 1] = parts
	end
	return items
end
local calls = {}
for line in string.gmatch (texts.rotor, "[^\n]+") do
	if string.sub (line, 1, #"env-rotor-call ") ~= "env-rotor-call " then
		fail ("setup_failure_evidence_incoherent", "SETUP FAILURE: the probe record holds a line that is not a call: " .. string.sub (line, 1, 80))
	end
	local f = fields_of (line)
	local ok, acts, kids = pcall (function () return list_of (f.act or "", "^([01]):(%d+):(%x%x%x%x%x%x%x%x)$", "a request"), list_of (f.kids or "", "^(%d+):(%d+):(%x+):(%x+):(%x+):(%x+):([01])$", "a child") end)
	if not ok or tonumber (f.seq) == nil or tonumber (f.phase) == nil or tonumber (f.player) == nil or tonumber (f.fd) == nil then
		fail ("setup_failure_evidence_incoherent", "SETUP FAILURE: a probe call is not in the delegate's grammar: " .. string.sub (line, 1, 160))
	end
	calls [#calls + 1] = { f = f, seq = tonumber (f.seq), phase = tonumber (f.phase), player = tonumber (f.player), fd = tonumber (f.fd), acts = acts, kids = kids }
end
local rows = {}
for index, kids in string.gmatch (texts.sweep, "env%-rotor%-out i=(%d+) kids=(%S+)") do
	local ok, items = pcall (list_of, kids, "^(%d+):(%x+):(%x+)$", "a sweep output")
	if not ok then fail ("setup_failure_evidence_incoherent", "SETUP FAILURE: a sweep output row is not in the delegate's grammar: " .. kids) end
	rows [#rows + 1] = { index = tonumber (index), kids = items }
end

local facts = string.format ('"calls":%d,', #calls)
if #calls < 1 then
	fail ("setup_failure_no_calls", "SETUP FAILURE: the probe recorded no call of the function, so nothing was exercised")
end
if requests == 0 then
	fail ("rotor_host_capability_missing", "rotor-host-capability-missing: the delegate completed on " .. #calls .. " call(s) of the original function, and the host was asked 0 times for 'rotor_sound' (" .. forwarded .. " other requests were forwarded unchanged): the campaign computed the rotor acoustics itself", true, facts)
end
if requests ~= #calls then
	fail ("ownership_not_shown", "ownership-not-shown: the function was called " .. #calls .. " time(s) and the host was asked " .. requests .. " time(s) for 'rotor_sound': the campaign did not ask the host exactly once for every call", true, facts)
end

-- ---------------------------------------------------------------------------------------------------------------- the joins
local ROLE = { [0] = "ai", [1] = "local", [2] = "remote" }
local CHANNELS = { "looping", "slap", "turbine1", "turbine2", "apu", "winding" }
-- the real sub-types (en_sbtyp.h) of the sound children and of the requests: the delegate's own enumerations
local SUB_OF = { looping = 33, apu = 34, turbine1 = 35, turbine2 = 36, slap = 37 }
local CHANNEL_OF_SUB = { [33] = "looping", [34] = "apu", [35] = "turbine1", [36] = "turbine2", [37] = "slap", [38] = "winding", [39] = "winding", [40] = "winding", [41] = "winding" }
local SOUND_EFFECT_TYPE = 32

local function evidence_problem (message) error ({ evidence = message }, 0) end
local function not_shown (message) error ({ shown = message }, 0) end

-- the words of an answer, or an evidence problem when a number is not an exact single or the shape is not the protocol's
local function words_of (answer, which)
	if type (answer) ~= "table" or type (answer.channels) ~= "table" or type (answer.actions) ~= "table" then evidence_problem ("the " .. which .. " answer is not a table with channels and actions") end
	local words = { amp = {}, pitch = {}, acts = {} }
	for _, name in ipairs (CHANNELS) do
		local channel = answer.channels [name]
		if type (channel) ~= "table" then evidence_problem ("the " .. which .. " answer has no channel " .. name) end
		words.amp [name] = word_of (channel.amplification)
		if words.amp [name] == nil then evidence_problem ("the " .. which .. " amplification of " .. name .. " is not an exact single") end
		if name ~= "winding" then
			words.pitch [name] = word_of (channel.pitch)
			if words.pitch [name] == nil then evidence_problem ("the " .. which .. " pitch of " .. name .. " is not an exact single") end
		end
	end
	for position, action in ipairs (answer.actions) do
		local sub = SUB_OF [action.channel]
		if sub == nil or (action.action ~= "pause" and action.action ~= "resume") then evidence_problem ("the " .. which .. " request " .. position .. " is not a pause or resume of a channel") end
		local delay = action.delay == nil and ZERO or word_of (action.delay)
		if delay == nil then evidence_problem ("the " .. which .. " delay of request " .. position .. " is not an exact single") end
		words.acts [position] = { resume = action.action == "resume", channel = action.channel, sub = sub, delay = delay }
	end
	return words
end

local function verify ()
local totals = { children = 0, looping = 0, slap = 0, others = 0, requests = 0, pauses = 0, sweep_pauses = 0, differing = 0, sweep_calls = 0 }
local phase1 = {}
if #log ~= #calls then evidence_problem ("the host answered " .. #log .. " requests for " .. #calls .. " calls") end
for k, call in ipairs (calls) do
	local entry = log [k]
	local what = "call " .. (k - 1) .. " (seq " .. call.seq .. ", phase " .. call.phase .. ")"
	if call.seq ~= k - 1 then evidence_problem (what .. " is not at its position in the probe record") end
	-- the request SENT carries what the function read
	local sent = entry.sent
	if sent.player ~= ROLE [call.player] then not_shown (what .. ": the request names the role " .. tostring (sent.player) .. " where the call is for player " .. call.player) end
	if (sent.flight_model == true) ~= (call.fd == 1) then not_shown (what .. ": the request's flight model flag is " .. tostring (sent.flight_model) .. " where the call recorded " .. call.fd) end
	if word_of (sent.rpm) ~= call.f.rpm_w then not_shown (what .. ": the request's rpm is " .. tostring (word_of (sent.rpm)) .. " where the call recorded " .. tostring (call.f.rpm_w)) end
	local acceleration, engines = sent.acceleration, sent.flight_model == true and sent.player == "local"
	if call.fd == 1 then
		if type (acceleration) ~= "table" then not_shown (what .. ": the request carries no acceleration though a flight model is current") end
		for _, pair in ipairs ({ { "x", "acc_x_w" }, { "y", "acc_y_w" }, { "z", "acc_z_w" } }) do
			if word_of (acceleration [pair [1]]) ~= call.f [pair [2]] then not_shown (what .. ": the request's acceleration " .. pair [1] .. " is not the recorded word " .. call.f [pair [2]]) end
		end
	elseif acceleration ~= nil then
		not_shown (what .. ": the request carries an acceleration with no flight model")
	end
	if engines then
		for _, pair in ipairs ({ { "left_rpm", "lrpm_w" }, { "right_rpm", "rrpm_w" }, { "apu_rpm", "apu_w" } }) do
			if word_of (sent [pair [1]]) ~= call.f [pair [2]] then not_shown (what .. ": the request's " .. pair [1] .. " is not the recorded word " .. call.f [pair [2]]) end
		end
	elseif sent.left_rpm ~= nil or sent.right_rpm ~= nil or sent.apu_rpm ~= nil then
		not_shown (what .. ": the request carries engine rpm that the original reads only for the local player with a flight model")
	end

	local returned, reference = words_of (entry.returned, "returned"), words_of (entry.reference, "reference")
	-- the CHANGED answer is exactly the reference plus the three level changes and the pause delays
	if mode == "changed" then
		local target = { looping_amp = word_of (CHANGED.looping_amplification), looping_pitch = word_of (CHANGED.looping_pitch), slap_amp = word_of (CHANGED.slap_amplification), delay = word_of (CHANGED.delay) }
		for _, name in ipairs (CHANNELS) do
			local want_amp = name == "looping" and target.looping_amp or name == "slap" and target.slap_amp or reference.amp [name]
			local want_pitch = name == "looping" and target.looping_pitch or reference.pitch [name]
			if returned.amp [name] ~= want_amp or returned.pitch [name] ~= want_pitch then not_shown (what .. ": the answer sent back for " .. name .. " is not the reference with only the intended changes") end
		end
		if #returned.acts ~= #reference.acts then not_shown (what .. ": the answer sent back has another number of requests than the reference") end
		for position, act in ipairs (returned.acts) do
			local ref = reference.acts [position]
			if act.resume ~= ref.resume or act.channel ~= ref.channel or act.delay ~= (act.resume and ref.delay or target.delay) then not_shown (what .. ": request " .. position .. " sent back is not the reference with only the pause delay changed") end
		end
		if returned.amp.looping ~= reference.amp.looping or returned.pitch.looping ~= reference.pitch.looping or returned.amp.slap ~= reference.amp.slap then totals.differing = totals.differing + 1 end
	end

	-- the recorded requests are the RETURNED actions: count, order, kind, sub-type and delay word
	if #call.acts ~= #returned.acts then not_shown (what .. ": the call recorded " .. #call.acts .. " pause or resume requests where the host answered " .. #returned.acts) end
	for position, act in ipairs (call.acts) do
		local want = returned.acts [position]
		if (act [1] == "1") ~= want.resume or tonumber (act [2]) ~= want.sub or act [3] ~= want.delay then
			not_shown (what .. ": request " .. position .. " is " .. act [1] .. ":" .. act [2] .. ":" .. act [3] .. " where the host answered " .. (want.resume and "1" or "0") .. ":" .. want.sub .. ":" .. want.delay)
		end
		totals.requests = totals.requests + 1
		if not want.resume then
			totals.pauses = totals.pauses + 1
			if call.phase == 1 then totals.sweep_pauses = totals.sweep_pauses + 1 end
		end
	end

	-- EVERY sound child, duplicates included, shows the returned answer; what the original does not write is unchanged from the recorded before word
	local sounds, looping, slap = {}, 0, 0
	for position, kid in ipairs (call.kids) do
		if tonumber (kid [1]) == SOUND_EFFECT_TYPE then
			local sub = tonumber (kid [2])
			local name = CHANNEL_OF_SUB [sub]
			local before_amp, before_pitch, after_amp, after_pitch = kid [3], kid [4], kid [5], kid [6]
			local label = what .. " child " .. position .. " (sub-type " .. sub .. ")"
			if name then
				if after_amp ~= returned.amp [name] then not_shown (label .. ": the amplification is " .. after_amp .. " where the host answered " .. returned.amp [name] .. " for " .. name) end
				if name == "winding" then
					if after_pitch ~= before_pitch then not_shown (label .. ": the winding pitch changed from " .. before_pitch .. " to " .. after_pitch .. " where the channel writes none") end
				elseif after_pitch ~= returned.pitch [name] then
					not_shown (label .. ": the pitch is " .. after_pitch .. " where the host answered " .. returned.pitch [name] .. " for " .. name)
				end
				if name == "looping" then looping = looping + 1 elseif name == "slap" then slap = slap + 1 end
			else
				if after_amp ~= before_amp or after_pitch ~= before_pitch then not_shown (label .. ": a sub-type the original does not handle was written (" .. before_amp .. ":" .. before_pitch .. " -> " .. after_amp .. ":" .. after_pitch .. ")") end
				totals.others = totals.others + 1
			end
			totals.children = totals.children + 1
			sounds [#sounds + 1] = { sub = kid [2], amp = after_amp, pitch = after_pitch }
		end
	end
	totals.looping, totals.slap = totals.looping + looping, totals.slap + slap
	if call.phase == 1 then
		phase1 [#phase1 + 1] = { call = call, sounds = sounds, what = what }
		-- a changed sweep call needs both targets to be witnessed by a child
		if mode == "changed" and (looping < 1 or slap < 1) then evidence_problem (what .. " has " .. looping .. " looping and " .. slap .. " slap children: the changed targets were not witnessed") end
	end
end
-- every row of the sweep output is joined to its phase-1 call and shows the same words, child for child
if #rows ~= #phase1 or #rows < 1 then evidence_problem ("the sweep output holds " .. #rows .. " rows for " .. #phase1 .. " phase-1 calls") end
for k, row in ipairs (rows) do
	local joined = phase1 [k]
	if row.index ~= k - 1 then evidence_problem ("the sweep output row " .. k .. " is numbered " .. row.index) end
	if #row.kids ~= #joined.sounds then not_shown (joined.what .. ": the sweep row lists " .. #row.kids .. " sound children where the call recorded " .. #joined.sounds) end
	for position, kid in ipairs (row.kids) do
		local sound = joined.sounds [position]
		if kid [1] ~= sound.sub or kid [2] ~= sound.amp or kid [3] ~= sound.pitch then
			not_shown (joined.what .. ": sweep row child " .. position .. " is " .. kid [1] .. ":" .. kid [2] .. ":" .. kid [3] .. " where the call recorded " .. sound.sub .. ":" .. sound.amp .. ":" .. sound.pitch)
		end
	end
	totals.sweep_calls = totals.sweep_calls + 1
end
if mode == "changed" and (totals.sweep_pauses < 1 or totals.differing < 1) then
	evidence_problem ("the sweep recorded " .. totals.sweep_pauses .. " pauses and the changed answers differed from the reference in " .. totals.differing .. " calls: the changed targets were not exercised")
end
return totals
end

local verified, outcome = pcall (verify)
if not verified then
	if type (outcome) == "table" and outcome.shown then
		fail ("ownership_not_shown", "ownership-not-shown: " .. outcome.shown, true, facts)
	end
	fail ("setup_failure_evidence_incoherent", "SETUP FAILURE: the delegate's artifacts cannot be joined to the host's requests: " .. (type (outcome) == "table" and tostring (outcome.evidence) or tostring (outcome)))
end
local shown = string.format ('"calls":%d,"children_checked":%d,"looping_children":%d,"other_children":%d,"pauses":%d,"requests_checked":%d,"slap_children":%d,"sweep_rows":%d,',
	#calls, outcome.children, outcome.looping, outcome.others, outcome.pauses, outcome.requests, outcome.slap, outcome.sweep_calls)
record (true, mode == "changed" and "host_changed_the_acoustics" or "host_answer_shown_by_every_child", shown)
io.stderr:write (string.format ("%s: mode=%s %d call(s), %d host request(s); %d sound children and %d requests show the host's answer; %d sweep rows joined\n", NAME, mode, #calls, requests, outcome.children, outcome.requests, outcome.sweep_calls))
