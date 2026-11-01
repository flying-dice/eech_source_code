-- TEST-ONLY PUBLIC-HOST OWNERSHIP WITNESS of the physical SOUND BACKEND AVAILABILITY at initialisation (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. A script only: no C, no ABI, no provider and no host solver is written here.
--
-- THE QUESTION. The original initialisation asked the platform sound module (`initialise_sound_system ()`, here the headless build's null DirectSound) whether a physical backend could be started and stored the answer as
-- `application_sound_system_enabled`. That decision is now the HOST's: behind its own flag test the campaign asks once, at boot, through the public path (host-owned Lua -> `eech_dc` -> the campaign -> the host's provider
-- callback) the operation `sound_backend` and takes the answer (docs/d3-sound-lifetime.md, eech-world/src/env.rs). This witness asks, on the native it is run against, whether the campaign really asks the host and whether the
-- host's ANSWER is what the engine acts on.
--
-- WHAT IT DOES. It wraps the `host.env_provider` factory BEFORE the delegate boots and counts ONLY the requests whose `op` is `sound_backend`; every other request is forwarded UNCHANGED to the original provider. It then
-- runs the BACKEND-INITIALISATION driver `lua/env-sound-init-backend.lua` (derived from the metadata driver, which refuses its report by design) through `dofile`, so the delegate's argument checks, report validation, raw
-- sidecar and output-failure handling are its own, with no second parser. Its report keeps three kinds of fact apart, and so does this witness: the HOST'S ANSWER (the `env-sndinit-host-backend` row, a mirror of what the
-- host said), the campaign's OWN device calls (the `env-sndinit-com` rows, which must be 0 because the campaign makes none) and the campaign's own stored flag (`enabled_backend`). The delegate's evidence stays on `out` and
-- `<out>.case15.raw` untouched; this witness writes ONE compact separate record of its own on `authority_out`.
--
-- MODES (`mode=`, default healthy):
--   healthy        the request is forwarded to the reference host, which has no audio device and answers `available = false`. REQUIRED: the delegate completed, the host was asked exactly once, the real report shows the host's
--                  answer (one answer, available 0), the campaign's stored flag equal to it (enabled_backend 0) and EVERY DirectSound entry counter of the campaign 0.
--   available      the witness replaces the host's answer by `available = true` (after the reference provider answered). The campaign has no handles for a backend and does not fake one: REQUIRED: it STOPS at
--                  `[sound-backend-provider:unsupported]` (one request, one altered answer, no provider error, empty evidence). This shows the host's answer changes the engine's outcome; it is a REFUSAL, never a successful
--                  positive backend.
--   withheld       the witness itself withdraws the answer: it records the withdrawal and raises its OWN identified error (the cause token `WITHDRAWAL`) BEFORE the reference provider is consulted, so an error of the provider
--                  can never be taken for it. REQUIRED: one request and one withdrawal, no provider error, the campaign STOPS at `[sound-backend-provider:callback]` AND the stop carries the withdrawal's cause, and the evidence is empty.
--   malformed      the host answers `available = "yes"` (not a boolean). REQUIRED: one request and one altered answer, no provider error, the campaign stops at `[sound-backend-provider:answer]`, and the evidence is empty.
--   nothing is stored before the answer is checked: a stop is the first and only thing the engine does with a refused answer, so the delegate writes no boot row.
--
-- THE EVIDENCE OF A REFUSAL. For every refusal the delegate's `out` and its raw sidecar are read back with a CHECKED open, read and close, and both must be EMPTY: the delegate creates them before boot and writes nothing until boot
-- has returned. A file that cannot be opened, read or closed is a SETUP FAILURE (evidence unavailable) and never proof that the campaign wrote no row; a boot row is a contradiction; any other text is unexpected evidence.
--
-- THE DELEGATE. `delegate=` is the backend driver on a native that has this exchange. On a native that does not (the metadata native 7c, the original 7574), the host is asked 0 times and the witness reaches the capability
-- verdict BEFORE it reads any hosted row, provided the delegate COMPLETES there: use `lua/env-sound-init-host.lua` on the metadata native and the ORIGINAL `lua/env-sound-init.lua` on the original (the backend driver refuses
-- their reports, by its own guards, before this witness could judge). The record names the delegate (`delegate`).
--
-- VERDICTS (errors begin `env-sound-backend-host-witness:`): `SETUP FAILURE` -- the delegate failed (healthy), the stop is not the engine's or not the intended refusal's (the reference provider raised first, the withdrawal is
-- not the cause, the host was never asked), or the evidence cannot be read back: NOT a verdict on ownership; `sound-backend-host-capability-missing`; `ownership-not-shown` -- the host was asked but the real report does not show
-- its answer; `not-stopped` / `wrong-boundary` -- a refused answer did not stop the campaign at its marker; `contradictory-result` / `unexpected-evidence` -- a stop that left a boot row or text.
--
-- WHAT IT DOES NOT. It asserts no private type, struct layout or ABI, creates no playback handle or device, loads no sample and exercises no renderer, speaker, network or playback route; it says nothing about the sample loader
-- bodies, the blocked-sample state, shutdown, the options menu, the per-frame scan or any playback duty, which stay Core. It computes nothing.
--
-- ARGUMENTS (host.args, as the host executable passes them):
--   eech-world <this script> root=<installation root> out=<delegate JSON Lines path> delegate=<ABSOLUTE path of lua/env-sound-init-backend.lua> [mode=healthy|available|withheld|malformed]
--                                                                       [scenario=lebanon_retail|georgia_retail] [seconds=<n>] [frame_ms=<n>] [authority_out=<this witness's record path; default <out>.authority.jsonl>]
--   `root`, `out`, `scenario`, `seconds` and `frame_ms` are the delegate's own arguments and are passed through unchanged. One fresh process and root per run.

io.stdout:setvbuf ("no")

local NAME = "env-sound-backend-host-witness"
local args = host.args
local function refuse (message) error (NAME .. ": " .. message, 0) end

local delegate_path = args.delegate
if type (delegate_path) ~= "string" or delegate_path == "" then refuse ("delegate=<absolute path of env-sound-init-backend.lua> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<the delegate's JSON Lines path> is required") end
local mode = args.mode == nil and "healthy" or args.mode
local MODES = {
	healthy = {},
	available = { marker = "[sound-backend-provider:unsupported]" },
	withheld = { marker = "[sound-backend-provider:callback]" },
	malformed = { marker = "[sound-backend-provider:answer]" },
}
local plan = MODES [mode]
if plan == nil then refuse ("mode must be healthy, available, withheld or malformed, got '" .. tostring (mode) .. "'") end
local MARKER = plan.marker
local OP = "sound_backend"
local authority_path = args.authority_out
if authority_path == nil then authority_path = out_path .. ".authority.jsonl" end
if type (authority_path) ~= "string" or authority_path == "" or authority_path == out_path then refuse ("authority_out must be a path other than out") end
local raw_path = args.raw_out
if raw_path == nil then raw_path = out_path .. ".case15.raw" end

-- ---------------------------------------------------------------------------------------------------------------- the wrapper: forward everything, count only the op, alter only that op's answer
local requests, forwarded, altered = 0, 0, 0
local withdrawals, base_errors = 0, 0
-- the cause of a withheld answer: raised only by this witness, before the reference provider is consulted, and required in the campaign's stop
local WITHDRAWAL = NAME .. ": the intentional withdrawal of the " .. OP .. " answer"
local factory = host.env_provider
if type (factory) ~= "function" then refuse ("this host has no host.env_provider factory") end

local function alter (answer)
	if mode == "available" then
		altered = altered + 1
		return { available = true }
	elseif mode == "malformed" then
		altered = altered + 1
		return { available = "yes" }
	end
	return answer
end

local wrapped = function (...)
	local base = factory (...)
	if type (base) ~= "function" then error (NAME .. ": host.env_provider () did not return a provider function", 0) end
	return function (request, ...)
		if type (request) == "table" and request.op == OP then
			requests = requests + 1
			-- WITHHELD: the withdrawal is this witness's own and is recorded and identified BEFORE the reference provider is consulted, so an error of that provider cannot be taken for it
			if mode == "withheld" then
				withdrawals = withdrawals + 1
				error (WITHDRAWAL, 0)
			end
			-- every other mode starts from the provider's answer: a provider that raises first never reaches the alteration, and that is counted, not mistaken for the intended refusal
			local consulted, answer = pcall (base, request, ...)
			if not consulted then
				base_errors = base_errors + 1
				error (answer, 0)
			end
			return alter (answer)
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

-- a CHECKED read of an evidence file: its text, or nil and the reason (a failed open, read or close is never an empty file)
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

local function json_text (value)
	return '"' .. (string.gsub (tostring (value), '[%c"\\\128-\255]', function (c) return string.format ("\\u%04x", string.byte (c)) end)) .. '"'
end

local function write_record (completed, verdict, extra)
	local file, err = host.output_create_new (authority_path)
	if not file then refuse ("OUTPUT FAILURE: creating " .. authority_path .. " failed: " .. tostring (err) .. " (evidence is created exclusively and never overwritten)") end
	local line = string.format ('{"altered":%d,"base_errors":%d,"completed":%s,"delegate":%s,"delegate_ran":%s,"forwarded_other_requests":%d,"mode":"%s","op":"%s","requests":%d,%s"source":"host.env_provider wrapper on the public path","verdict":"%s","withdrawals":%d}\n',
		altered, base_errors, completed and "true" or "false", json_text (delegate_path), ran and "true" or "false", forwarded, mode, OP, requests, extra or "", verdict, withdrawals)
	local ok, werr = file:write (line)
	if not ok then refuse ("OUTPUT FAILURE: writing " .. authority_path .. " failed: " .. tostring (werr)) end
	ok, werr = file:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. authority_path .. " failed: " .. tostring (werr)) end
	ok, werr = file:close ()
	if not ok then refuse ("OUTPUT FAILURE: closing " .. authority_path .. " failed: " .. tostring (werr)) end
end

-- one row of the delegate's raw native report, as `key=value` fields (nil when absent)
local function fields_of (text, prefix)
	for line in string.gmatch (text, "[^\n]+") do
		if string.sub (line, 1, #prefix) == prefix then
			local map = {}
			for key, value in string.gmatch (line, "([%w_]+)=(%S+)") do map [key] = value end
			return map
		end
	end
	return nil
end

local function has (text, fragment) return text ~= nil and string.find (text, fragment, 1, true) ~= nil end

-- ---------------------------------------------------------------------------------------------------------------- available, withheld, malformed: the campaign must STOP at the named boundary
if MARKER then
	if ran then
		write_record (true, "not_stopped")
		refuse ("not-stopped: the host answered its '" .. OP .. "' request wrongly (" .. mode .. ", " .. requests .. " request(s)) and the delegate completed: the campaign did not stop at " .. MARKER)
	end
	local delegate_error = tostring (problem)
	-- the evidence the stop is judged by: the delegate's out and its raw sidecar, read back with a checked open, read and close. Unavailable evidence is a failure of the witness, never "no row".
	local evidence = {}
	for _, item in ipairs ({ { "out", out_path }, { "sidecar", raw_path } }) do
		local text, why = read_checked (item [2])
		if text == nil then
			write_record (false, "setup_failure_evidence_unreadable")
			refuse ("SETUP FAILURE: the delegate's evidence (" .. item [1] .. ") " .. item [2] .. " could not be read (" .. why .. "): a refusal is judged by what the campaign left behind, and there is no verdict without it")
		end
		evidence [item [1]] = text
	end
	if requests == 0 then
		write_record (false, "setup_failure_the_host_was_never_asked")
		refuse ("SETUP FAILURE: the delegate stopped but the host was asked 0 times for '" .. OP .. "': the stop is not the " .. mode .. " answer's: " .. delegate_error)
	end
	if base_errors > 0 then
		write_record (false, "setup_failure_base_provider_error")
		refuse ("SETUP FAILURE: the reference provider raised an error " .. base_errors .. " time(s) before the " .. mode .. " refusal could be made: the stop is the provider's, not the intended refusal's: " .. delegate_error)
	end
	if mode == "withheld" then
		if withdrawals ~= 1 or requests ~= 1 then
			write_record (false, "setup_failure_no_single_withdrawal")
			refuse ("SETUP FAILURE: the host recorded " .. withdrawals .. " withdrawals in " .. requests .. " request(s) where the refusal is one withdrawal of one request: " .. delegate_error)
		end
	elseif altered ~= 1 or requests ~= 1 then
		write_record (false, "setup_failure_nothing_was_altered")
		refuse ("SETUP FAILURE: the host altered " .. altered .. " answers in " .. requests .. " request(s) in mode " .. mode .. " where the refusal is one altered answer of one request: the stop is not the altered answer's: " .. delegate_error)
	end
	if not has (delegate_error, "host sound backend") then
		write_record (false, "setup_failure_delegate_error_is_not_the_engine_stop")
		refuse ("SETUP FAILURE: the delegate's error is not its report of the engine's stop at the sound backend provider: " .. delegate_error)
	end
	if not has (delegate_error, MARKER) then
		write_record (false, "wrong_boundary")
		refuse ("wrong-boundary: the engine stopped at the sound backend provider, but not at " .. MARKER .. ": " .. delegate_error)
	end
	-- the generic callback marker is not the cause: the stop must carry this witness's own withdrawal
	if mode == "withheld" and not has (delegate_error, WITHDRAWAL) then
		write_record (false, "setup_failure_withdrawal_not_the_cause")
		refuse ("SETUP FAILURE: the engine stopped at " .. MARKER .. " but the stop does not carry this witness's withdrawal (" .. WITHDRAWAL .. "): the callback failed for another reason: " .. delegate_error)
	end
	if has (evidence.out, '"event":"boot"') then
		write_record (false, "contradictory_boot_row")
		refuse ("contradictory-result: the delegate wrote its boot row although the campaign is said to have stopped inside boot")
	end
	if evidence.out ~= "" or evidence.sidecar ~= "" then
		write_record (false, "unexpected_evidence_state")
		refuse ("unexpected-evidence: the campaign stopped inside boot but its evidence is not empty (out " .. #evidence.out .. " bytes, sidecar " .. #evidence.sidecar .. " bytes)")
	end
	write_record (false, "stopped_at_boundary", string.format ('"marker":"%s","stopped":true,', MARKER))
	io.stderr:write (string.format ("%s: mode=%s stopped at %s after %d host request(s) for '%s'\n", NAME, mode, MARKER, requests, OP))
	return
end

-- ---------------------------------------------------------------------------------------------------------------- healthy: the delegate must have completed, and the real report must show the host's answer
if not ran then
	write_record (false, "setup_failure_delegate_failed")
	error (tostring (problem), 0)
end
if requests == 0 then
	write_record (true, "sound_backend_host_capability_missing")
	refuse ("sound-backend-host-capability-missing: the delegate completed and the host was asked 0 times for '" .. OP .. "': the campaign still decides its sound backend itself")
end
local raw, raw_why = read_checked (raw_path)
if raw == nil then
	write_record (false, "setup_failure_evidence_unreadable")
	refuse ("SETUP FAILURE: the delegate's raw native report " .. raw_path .. " could not be read (" .. raw_why .. "): there is no ownership verdict")
end
local boot = fields_of (raw, "env-sndinit-boot n=0")
local answered = fields_of (raw, "env-sndinit-host-backend n=0")
local coms = {}
for _, name in ipairs ({ "DirectSoundCreate", "IDirectSound_SetCooperativeLevel", "IDirectSound_CreateSoundBuffer", "IDirectSound_Release" }) do
	coms [#coms + 1] = { name, fields_of (raw, "env-sndinit-com function=" .. name) }
end
local complete = boot and answered
for _, item in ipairs (coms) do complete = complete and item [2] end
if not complete then
	write_record (false, "setup_failure_no_report")
	refuse ("SETUP FAILURE: the delegate returned but its native report (" .. raw_path .. ") has no boot, host-backend and DirectSound rows: there is no ownership verdict")
end
local wrong = {}
local function expect (what, wanted, observed)
	if wanted ~= observed then wrong [#wrong + 1] = what .. " expected " .. tostring (wanted) .. " observed " .. tostring (observed) end
end
-- the HOST's answer: one answer, available 0, mirrored; the campaign's stored flag is that answer; and the campaign made no device call, which is a separate fact from the host's answer
expect ("host_reply_answers", "1", answered.answers)
expect ("host_reply_available", "0", answered.available)
expect ("source", "host_reply_mirror", answered.source)
expect ("enabled_backend", answered.available, boot.enabled_backend)
expect ("initialised_backend", "0", boot.initialised_backend)
for _, item in ipairs (coms) do
	expect ("core_" .. item [1] .. "_calls", "0", item [2].calls)
end
if requests ~= 1 then wrong [#wrong + 1] = "the host was asked " .. requests .. " times where the campaign asks once" end
if base_errors ~= 0 then wrong [#wrong + 1] = "the reference provider raised " .. base_errors .. " time(s)" end
if #wrong > 0 then
	write_record (true, "ownership_not_shown")
	refuse ("ownership-not-shown: the host was asked " .. requests .. " time(s) but the real report does not show its answer: " .. table.concat (wrong, "; "))
end
write_record (true, "host_asked_and_decided")
io.stderr:write (string.format ("%s: mode=healthy: the host was asked %d time(s) and the real report shows its answer (available %s) with no campaign device call\n", NAME, requests, answered.available))
