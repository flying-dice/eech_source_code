-- TEST-ONLY PUBLIC-HOST OWNERSHIP WITNESS of the helicopter rotor acoustics (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. A script only: no C, no ABI, no provider and no host solver is written here.
--
-- THE QUESTION. The original `update_local_helicopter_rotor_sounds` computed every rotor, slap, turbine and APU level and pitch, and chose the pause and resume requests, itself. That arithmetic and that choice are
-- now the HOST's: the campaign reaches the host through the public path (host-owned Lua -> `eech_dc` -> the campaign -> the host's provider callback) as the physical operation `rotor_sound`, and the host answers the
-- levels and pitches of the logical channels and the ordered requests (docs/d3-sound-lifetime.md and eech-world/src/env.rs have the request and the reply). This witness asks, on the native it is run against, whether
-- the campaign really asks the host once for every call of the function and whether the host's ANSWER is what the real sound children and the real pause requests show.
--
-- WHAT IT DOES. It wraps the `host.env_provider` factory BEFORE the delegate boots and counts ONLY the requests whose `op` is `rotor_sound`; every other request is forwarded UNCHANGED to the original provider. What it
-- does with a `rotor_sound` request depends on the mode. It then runs the EXISTING original-rotor driver `lua/env-rotor-sounds.lua` (default `sweep=ai`, `seconds=0`) through `dofile`, so the delegate's argument checks,
-- strict record grammar, raw sidecars and exclusive-output handling are exactly its own, with no second parser. The delegate's rows stay on `out` untouched; this witness writes ONE compact separate record of its own
-- on `authority_out`.
--
-- MODES (`mode=`, default healthy):
--   healthy    the request is answered by the reference host unchanged. REQUIRED: the delegate completed and the host was asked EXACTLY once for every call of the function the probe recorded (`<out>.rotor.raw`).
--              Against the pre-extraction native (for instance the original measured native e54423b5c) the campaign computed the levels itself and the host was asked 0 times: the error
--              `rotor-host-capability-missing` (the RED of the capability, not a physical regression).
--   changed    the reference host answers, and the witness then CHANGES the answer in a valid way: the looping channel's level and pitch, the slap channel's level and the delay of every pause. REQUIRED: the host was asked and the
--              REAL record shows the changed words: in `<out>.sweep.raw` every looping sound child holds the looping level and pitch the host answered and every slap child the slap level, the other children hold what the
--              reference host answered (not the changed words), and in `<out>.rotor.raw` every recorded sweep pause carries the delay word the host answered (and at least one pause exists). A request count alone proves
--              nothing: a native that asks the host and still computes the levels itself fails here.
--   withheld   the host raises an error. REQUIRED: the delegate FAILS (no result row) at `[env-provider:rotor_sound.callback]` after exactly one host request.
--   malformed  the host answers the reference answer with a pitch on the winding channel, which writes an amplification only. REQUIRED: the delegate fails (no result row) at `[env-provider:rotor_sound.answer]` and names the
--              winding channel, after exactly one host request.
-- A stop is the expected evidence of withheld and malformed, not a completed success and not a fallback: the first request of the function is made while the engine boots, so the delegate's own error carries the
-- campaign's marker and the delegate wrote no result row. A campaign that completes anyway, or stops at another marker, is an error.
--
-- VERDICTS (errors begin `env-rotor-host-witness:`): `SETUP FAILURE` -- the delegate failed (healthy, changed), left no probe record, or a stop was not the campaign's: NOT a verdict on ownership;
-- `rotor-host-capability-missing` -- the delegate completed and the host was asked 0 times; `ownership-not-shown` -- the host was asked but the real record does not show its answer (a wrong number of requests in
-- healthy mode, a child or a pause that does not show the changed words in changed mode); `not-stopped` / `wrong-boundary` -- a withheld or malformed answer did not stop the campaign at its marker.
--
-- WHAT IT DOES NOT. It asserts no private type, struct layout or ABI, picks no physical outcome beyond the reference host's own, exercises no renderer, speaker, sound backend or network, and proves no audibility. It
-- covers the AI (no flight model) shapes of `sweep=ai`; the flight model shapes are `sweep=fd` of the same delegate (pass `sweep=fd`; the healthy and stop modes apply unchanged, `changed` needs the looping and slap
-- children of that sweep, which it has). It says nothing of the other physical responsibilities of D3.
--
-- ARGUMENTS (host.args, as the host executable passes them; the witness is the SCRIPT the host runs and the delegate is loaded from a path):
--   eech-world <this script> root=<installation root> out=<delegate JSON Lines path> delegate=<ABSOLUTE path of lua/env-rotor-sounds.lua> [mode=healthy|changed|withheld|malformed] [sweep=ai|fd (default ai)]
--                                                                                  [scenario=lebanon_retail|georgia_retail] [authority_out=<this witness's record path; default <out>.authority.jsonl>]
--   `root`, `out`, `scenario` and `sweep` are the delegate's own arguments and are passed through unchanged (the delegate reads the same `host.args`; `seconds` and `probe` are the delegate's too, with `probe=off`
--   refused here because the probe's record is the evidence). One fresh process and root per run.

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
local authority_path = args.authority_out
if authority_path == nil then authority_path = out_path .. ".authority.jsonl" end
if type (authority_path) ~= "string" or authority_path == "" or authority_path == out_path then refuse ("authority_out must be a path other than out") end

-- the words of the changed answer (all exact single-precision values) and the words the delegate's records show for them
local CHANGED = {
	looping_amplification = 0.375, looping_amplification_word = "3ec00000",
	looping_pitch = 0.8125, looping_pitch_word = "3f500000",
	slap_amplification = 0.1875, slap_amplification_word = "3e400000",
	delay = 2.0, delay_word = "40000000",
}
local SUB_LOOPING, SUB_SLAP = 33, 37

-- ---------------------------------------------------------------------------------------------------------------- the wrapper: forward everything, count only the op, alter only that op's answer
local requests, forwarded = 0, 0
local factory = host.env_provider
if type (factory) ~= "function" then refuse ("this host has no host.env_provider factory") end

local wrapped = function (...)
	local base = factory (...)
	if type (base) ~= "function" then error (NAME .. ": host.env_provider () did not return a provider function", 0) end
	return function (request, ...)
		if type (request) == "table" and request.op == "rotor_sound" then
			requests = requests + 1
			if mode == "withheld" then error ("the host withholds its rotor_sound answer", 0) end
			local answer = base (request, ...)
			if mode == "malformed" then
				if type (answer) == "table" and type (answer.channels) == "table" and type (answer.channels.winding) == "table" then
					answer.channels.winding.pitch = 1.0
				end
				return answer
			end
			if mode == "changed" and type (answer) == "table" and type (answer.channels) == "table" then
				answer.channels.looping.amplification = CHANGED.looping_amplification
				answer.channels.looping.pitch = CHANGED.looping_pitch
				answer.channels.slap.amplification = CHANGED.slap_amplification
				for _, action in ipairs (answer.actions or {}) do
					if action.action == "pause" then action.delay = CHANGED.delay end
				end
			end
			return answer
		end
		forwarded = forwarded + 1
		return base (request, ...)
	end
end
local installed, install_error = pcall (function () host.env_provider = wrapped end)
if not installed or host.env_provider ~= wrapped then refuse ("the host table does not accept a wrapped env_provider: " .. tostring (install_error)) end

-- ---------------------------------------------------------------------------------------------------------------- the delegate, exactly as it is
local ran, problem = pcall (dofile, delegate_path)
host.env_provider = factory

local function read_all (path)
	local file, err = io.open (path, "r")
	if not file then return nil, err end
	local text = file:read ("*a")
	file:close ()
	return text
end
local function has (line, fragment) return string.find (line, fragment, 1, true) ~= nil end

local function write_record (completed, verdict, extra)
	local file, err = io.open (authority_path, "w")
	if not file then refuse ("OUTPUT FAILURE: opening " .. authority_path .. " failed: " .. tostring (err)) end
	local line = string.format ('{"completed":%s,"delegate_ran":%s,"forwarded_other_requests":%d,"mode":"%s","op":"rotor_sound","requests":%d,%s"source":"host.env_provider wrapper on the public path","sweep":"%s","verdict":"%s"}\n',
		completed and "true" or "false", ran and "true" or "false", forwarded, mode, requests, extra or "", args.sweep, verdict)
	local ok, werr = file:write (line)
	if not ok then refuse ("OUTPUT FAILURE: writing " .. authority_path .. " failed: " .. tostring (werr)) end
	ok, werr = file:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. authority_path .. " failed: " .. tostring (werr)) end
	ok, werr = file:close ()
	if not ok then refuse ("OUTPUT FAILURE: closing " .. authority_path .. " failed: " .. tostring (werr)) end
end

-- the delegate's OWN record of this run: its result row (true when it wrote one)
local function result_row ()
	local text = read_all (out_path)
	if text == nil then return false end
	for line in string.gmatch (text, "[^\n]+") do
		if has (line, '"event":"result"') then return true end
	end
	return false
end

-- ---------------------------------------------------------------------------------------------------------------- withheld and malformed: the campaign must STOP at the named boundary
if MARKER then
	if ran then
		write_record (result_row (), "not_stopped")
		refuse ("not-stopped: the host answered its 'rotor_sound' request wrongly (" .. mode .. ", " .. requests .. " request(s)) and the delegate completed: the campaign did not stop at " .. MARKER)
	end
	local delegate_error = tostring (problem)
	if requests == 0 then
		write_record (false, "setup_failure_the_host_was_never_asked")
		refuse ("SETUP FAILURE: the delegate stopped but the host was asked 0 times for 'rotor_sound': the stop is not the " .. mode .. " answer's: " .. delegate_error)
	end
	if result_row () then
		write_record (false, "contradictory_result_row")
		refuse ("contradictory-result: the delegate wrote a result row although the campaign is said to have stopped: " .. delegate_error)
	end
	if not has (delegate_error, MARKER) then
		write_record (false, "wrong_boundary")
		refuse ("wrong-boundary: the delegate stopped, but not at " .. MARKER .. ": " .. delegate_error)
	end
	if mode == "malformed" and not has (delegate_error, "winding") then
		write_record (false, "wrong_reason")
		refuse ("wrong-reason: the campaign stopped at " .. MARKER .. " but the refusal does not name the winding channel that was given a pitch: " .. delegate_error)
	end
	if requests ~= 1 then
		write_record (false, "setup_failure_more_than_one_request")
		refuse ("SETUP FAILURE: the campaign made " .. requests .. " host requests before it stopped where one was expected (the first request stops the engine)")
	end
	write_record (false, "stopped_at_boundary", string.format ('"marker":"%s","stopped":true,', MARKER))
	io.stderr:write (string.format ("%s: mode=%s stopped at %s after %d host request(s) for 'rotor_sound'\n", NAME, mode, MARKER, requests))
	return
end

-- ---------------------------------------------------------------------------------------------------------------- healthy and changed: the delegate must have completed and the real record must show the host's answer
if not ran then
	write_record (false, "setup_failure_delegate_failed")
	error (tostring (problem), 0)
end
local rotor_text = read_all (out_path .. ".rotor.raw")
if rotor_text == nil then
	write_record (false, "setup_failure_no_probe_record")
	refuse ("SETUP FAILURE: the delegate returned but left no probe record on " .. out_path .. ".rotor.raw: the function's calls are not known, so there is no ownership verdict")
end
local calls = 0
local pauses, wrong_delay = 0, 0
for line in string.gmatch (rotor_text, "[^\n]+") do
	if string.sub (line, 1, #"env-rotor-call ") == "env-rotor-call " then
		calls = calls + 1
		if has (line, " phase=1 ") then
			local acts = string.match (line, " act=(%S+)")
			for kind, delay in string.gmatch (acts or "", "(%d):%d+:(%x+)") do
				if kind == "0" then
					pauses = pauses + 1
					if delay ~= CHANGED.delay_word then wrong_delay = wrong_delay + 1 end
				end
			end
		end
	end
end
local facts = string.format ('"calls":%d,', calls)
if calls < 1 then
	write_record (false, "setup_failure_no_calls")
	refuse ("SETUP FAILURE: the probe recorded no call of the function, so nothing was exercised")
end
if requests == 0 then
	write_record (true, "rotor_host_capability_missing", facts)
	refuse ("rotor-host-capability-missing: the delegate completed on " .. calls .. " call(s) of the original function, and the host was asked 0 times for 'rotor_sound' (" .. forwarded .. " other requests were forwarded unchanged): the campaign computed the rotor acoustics itself")
end
if requests ~= calls then
	write_record (true, "ownership_not_shown", facts)
	refuse ("ownership-not-shown: the function was called " .. calls .. " time(s) and the host was asked " .. requests .. " time(s) for 'rotor_sound': the campaign did not ask the host exactly once for every call")
end

if mode == "changed" then
	local sweep_text = read_all (out_path .. ".sweep.raw")
	if sweep_text == nil then
		write_record (false, "setup_failure_no_sweep_record")
		refuse ("SETUP FAILURE: the delegate returned but left no sweep record on " .. out_path .. ".sweep.raw")
	end
	local rows, looping_shown, slap_shown, other_changed = 0, 0, 0, 0
	for _, kids in string.gmatch (sweep_text, "env%-rotor%-out i=(%d+) kids=(%S+)") do
		rows = rows + 1
		for sub, amplification, pitch in string.gmatch (kids, "(%d+):(%x+):(%x+)") do
			sub = tonumber (sub)
			if sub == SUB_LOOPING then
				if amplification == CHANGED.looping_amplification_word and pitch == CHANGED.looping_pitch_word then looping_shown = looping_shown + 1 end
			elseif sub == SUB_SLAP then
				if amplification == CHANGED.slap_amplification_word then slap_shown = slap_shown + 1 end
			elseif amplification == CHANGED.looping_amplification_word or pitch == CHANGED.looping_pitch_word then
				other_changed = other_changed + 1
			end
		end
	end
	local shown = string.format ('"calls":%d,"looping_children_showing_the_answer":%d,"pauses":%d,"slap_children_showing_the_answer":%d,"sweep_rows":%d,', calls, looping_shown, pauses, slap_shown, rows)
	if rows < 1 then
		write_record (false, "setup_failure_no_sweep_rows")
		refuse ("SETUP FAILURE: the sweep record holds no output row")
	end
	if pauses < 1 then
		write_record (false, "setup_failure_no_pause_in_the_sweep", shown)
		refuse ("SETUP FAILURE: the sweep recorded no pause, so the host's delay was not exercised")
	end
	-- every looping and slap child of every row must show the changed words: at least one per row, and none of them showing anything else (a child count is the delegate's, checked there)
	if looping_shown < rows or slap_shown < rows or other_changed > 0 or wrong_delay > 0 then
		write_record (true, "ownership_not_shown", shown)
		refuse ("ownership-not-shown: the host was asked " .. requests .. " time(s) and changed its answer, but the real record does not show it (" .. rows .. " sweep rows; looping children showing it " .. looping_shown .. ", slap children " .. slap_shown .. ", other children carrying the changed looping words " .. other_changed .. ", pauses with another delay " .. wrong_delay .. " of " .. pauses .. "): the campaign did not use the host's answer")
	end
	write_record (true, "host_changed_the_acoustics", shown)
	io.stderr:write (string.format ("%s: mode=changed %d call(s), %d host request(s); %d sweep rows show the changed looping and slap words; %d pause(s) carry the changed delay\n", NAME, calls, requests, rows, pauses))
	return
end

write_record (true, "host_asked_for_every_call", facts)
io.stderr:write (string.format ("%s: mode=healthy %d call(s) of the function, %d host request(s) for 'rotor_sound'; %d other request(s) forwarded unchanged\n", NAME, calls, requests, forwarded))
