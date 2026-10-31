-- TEST-ONLY PUBLIC-HOST OWNERSHIP WITNESS of the physical lifetime of a sound effect (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. A script only: no C, no ABI, no provider and no host solver is written here.
--
-- THE QUESTION. The original constructor summed the samples' durations and capped them, and the original server and client update routines aged the lifetime, decided the expiry and the kill, clamped the client
-- and counted an invalid effect down to its unlink, all inside the campaign. Those decisions are now the HOST's: they reach it through the public path (host-owned Lua -> `eech_dc` -> the campaign -> the host's
-- provider callback) as the physical operations `sound_lifetime_init` and `sound_lifetime_update` (docs/d3-sound-lifetime.md, eech-world/src/env.rs). This witness asks, on the native it is run against, whether the campaign
-- really asks the host and whether the host's ANSWER is what the real entity shows.
--
-- WHAT IT DOES. It wraps the `host.env_provider` factory BEFORE the delegate boots and counts ONLY the requests whose `op` is the mode's; every other request is forwarded UNCHANGED to the original provider. It
-- then runs the EXISTING control driver `lua/env-sound-lifetime.lua` for the mode's controlled-data case through `dofile`, so the delegate's argument checks, report validation, raw rows and output-failure
-- handling are exactly its own, with no second parser. The delegate's rows stay on `out` untouched; this witness writes ONE compact separate record of its own on `authority_out`.
--
-- MODES (`mode=`, default healthy):
--   healthy        the request is forwarded to the reference host. REQUIRED: the case completed, the host was asked at least once and the real case shows what the ORIGINAL measured for it (case 4, the server's
--                  aging of a positive lifetime: three calls and ONE kill). Against a native that decides in the campaign the host is asked 0 times: `sound-host-capability-missing` (the RED of the capability).
--   changed        (case 4, `sound_lifetime_update`) the host answers every update itself with a valid answer that differs from the reference: nothing ages and nothing expires. REQUIRED: the host was asked and the
--                  REAL case shows NO kill (three calls, zero kills). A request count alone proves nothing: a native that asks the host and still ages and kills in the campaign fails here.
--   changed_init   (case 6, `sound_lifetime_init`) the host gives the new effect the lifetime 0.25. REQUIRED: the host was asked and the REAL entity's constructor state holds exactly 0.25.
--   withheld       the host raises an error in `sound_lifetime_update`. REQUIRED: the campaign STOPS at `[env-provider:sound_lifetime_update.callback]` (the delegate fails in the engine, no result row).
--   malformed      the host answers an action that does not exist. REQUIRED: the campaign stops at `[env-provider:sound_lifetime_update.answer]`, likewise.
--   withheld_init  the host raises an error in `sound_lifetime_init`. REQUIRED: the campaign stops at `[env-provider:sound_lifetime_init.callback]`, likewise.
-- THE STOP EVIDENCE is the delegate's OWN record, read back when the delegate fails: the case's `engine_failure` row at the expected marker and NO result row, and the delegate's error must be its report of the
-- engine's stop. A failure whose message happens to carry the marker (a path, a diagnostic) is rejected as that failure, and a stale or contradictory result row is `contradictory-result`.
--
-- VERDICTS (errors begin `env-sound-host-witness:`): `SETUP FAILURE` -- the delegate failed (healthy, changed) or the case was refused or has no result: NOT a verdict on ownership; `sound-host-capability-missing`;
-- `ownership-not-shown` -- the host was asked but the real case does not show the host's answer; `not-stopped` / `wrong-boundary` -- a withheld or malformed answer did not stop the campaign at its marker.
--
-- WHAT IT DOES NOT. It asserts no private type, struct layout or ABI, creates no playback handle and exercises no renderer, speaker, network, pause/resume or device. The word values it checks are the observed ORIGINAL
-- outputs of the finite controlled-data cases, and the altered ones it writes itself; it computes no lifetime.
--
-- ARGUMENTS (host.args, as the host executable passes them):
--   eech-world <this script> root=<installation root> out=<delegate JSON Lines path> delegate=<ABSOLUTE path of lua/env-sound-lifetime.lua> [mode=healthy|changed|changed_init|withheld|malformed|withheld_init]
--                                                                                  [scenario=lebanon_retail|georgia_retail] [authority_out=<this witness's record path; default <out>.authority.jsonl>]
--   `root`, `out`, `scenario` are the delegate's own arguments and are passed through unchanged; `case` is selected by the mode. One fresh process and root per run.

io.stdout:setvbuf ("no")

local NAME = "env-sound-host-witness"
local args = host.args
local function refuse (message) error (NAME .. ": " .. message, 0) end

local delegate_path = args.delegate
if type (delegate_path) ~= "string" or delegate_path == "" then refuse ("delegate=<absolute path of env-sound-lifetime.lua> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<the delegate's JSON Lines path> is required") end
local mode = args.mode == nil and "healthy" or args.mode
local MODES = {
	healthy = { op = "sound_lifetime_update", case = 4 },
	changed = { op = "sound_lifetime_update", case = 4 },
	changed_init = { op = "sound_lifetime_init", case = 6 },
	withheld = { op = "sound_lifetime_update", case = 4, marker = "[env-provider:sound_lifetime_update.callback]" },
	malformed = { op = "sound_lifetime_update", case = 4, marker = "[env-provider:sound_lifetime_update.answer]" },
	withheld_init = { op = "sound_lifetime_init", case = 6, marker = "[env-provider:sound_lifetime_init.callback]" },
}
local plan = MODES [mode]
if plan == nil then refuse ("mode must be healthy, changed, changed_init, withheld, malformed or withheld_init, got '" .. tostring (mode) .. "'") end
local OP, CASE, MARKER = plan.op, plan.case, plan.marker
if args.case ~= nil and tostring (args.case) ~= tostring (CASE) then refuse ("case must be " .. CASE .. " for mode " .. mode .. ", got '" .. tostring (args.case) .. "'") end
args.case = tostring (CASE)
local authority_path = args.authority_out
if authority_path == nil then authority_path = out_path .. ".authority.jsonl" end
if type (authority_path) ~= "string" or authority_path == "" or authority_path == out_path then refuse ("authority_out must be a path other than out") end

-- ---------------------------------------------------------------------------------------------------------------- the wrapper: forward everything, count only the op, alter only that op's answer
local requests, forwarded = 0, 0
local factory = host.env_provider
if type (factory) ~= "function" then refuse ("this host has no host.env_provider factory") end

local wrapped = function (...)
	local base = factory (...)
	if type (base) ~= "function" then error (NAME .. ": host.env_provider () did not return a provider function", 0) end
	return function (request, ...)
		if type (request) == "table" and request.op == OP then
			requests = requests + 1
			-- the host never ages an effect and never expires one: the lifetimes come back as they went, nothing is decided
			if mode == "changed" then return { lifetime = request.lifetime, valid_lifetime = request.valid_lifetime, expired = false, action = "none" } end
			if mode == "changed_init" then return { lifetime = 0.25 } end
			if mode == "withheld" or mode == "withheld_init" then error ("the host withholds its " .. OP .. " answer", 0) end
			if mode == "malformed" then return { lifetime = 1, valid_lifetime = 0, expired = false, action = "explode" } end
		else
			forwarded = forwarded + 1
		end
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

local function write_record (completed, verdict, extra)
	local file, err = io.open (authority_path, "w")
	if not file then refuse ("OUTPUT FAILURE: opening " .. authority_path .. " failed: " .. tostring (err)) end
	local line = string.format ('{"case":%d,"completed":%s,"delegate_ran":%s,"forwarded_other_requests":%d,"mode":"%s","op":"%s","requests":%d,%s"source":"host.env_provider wrapper on the public path","verdict":"%s"}\n',
		CASE, completed and "true" or "false", ran and "true" or "false", forwarded, mode, OP, requests, extra or "", verdict)
	local ok, werr = file:write (line)
	if not ok then refuse ("OUTPUT FAILURE: writing " .. authority_path .. " failed: " .. tostring (werr)) end
	ok, werr = file:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. authority_path .. " failed: " .. tostring (werr)) end
	ok, werr = file:close ()
	if not ok then refuse ("OUTPUT FAILURE: closing " .. authority_path .. " failed: " .. tostring (werr)) end
end

-- the delegate's OWN record: the rows it wrote on `out`, read back even when the delegate failed (nil when the file cannot be read)
local function delegate_rows ()
	local text = read_all (out_path)
	if text == nil then return nil end
	local rows = {}
	for line in string.gmatch (text, "[^\n]+") do rows [#rows + 1] = line end
	return rows
end
local function has (line, fragment) return string.find (line, fragment, 1, true) ~= nil end

-- the delegate's result row of this mode's case, if it wrote one: (completed, calls, kills); and the constructor state's lifetime word of its first entity
local function case_result ()
	local rows = delegate_rows ()
	if rows == nil then return false end
	local creation_word
	for _, line in ipairs (rows) do
		if has (line, '"event":"env-sound-state"') and has (line, '"ent":"0"') and has (line, '"tag":"0"') then creation_word = string.match (line, '"life_w":"(%x+)"') end
	end
	for _, line in ipairs (rows) do
		if has (line, '"event":"result"') and has (line, '"case":' .. CASE .. ',') and has (line, '"refused":false') then
			return true, tonumber (string.match (line, '"calls":(%d+)')), tonumber (string.match (line, '"kills":(%d+)')), creation_word
		end
	end
	return false
end

-- ---------------------------------------------------------------------------------------------------------------- withheld and malformed: the campaign must STOP at the named boundary
if MARKER then
	if ran then
		write_record (case_result (), "not_stopped")
		refuse ("not-stopped: the host answered its '" .. OP .. "' request wrongly (" .. mode .. ", " .. requests .. " request(s)) and the delegate completed: the campaign did not stop at " .. MARKER)
	end
	local delegate_error = tostring (problem)
	if requests == 0 then
		write_record (false, "setup_failure_the_host_was_never_asked")
		refuse ("SETUP FAILURE: the delegate stopped but the host was asked 0 times for '" .. OP .. "': the stop is not the " .. mode .. " answer's: " .. delegate_error)
	end
	local rows = delegate_rows ()
	if rows == nil then
		write_record (false, "setup_failure_no_delegate_record")
		refuse ("SETUP FAILURE: the delegate failed and left no readable record on " .. out_path .. ": " .. delegate_error)
	end
	local failure_row, result_row = nil, nil
	for _, line in ipairs (rows) do
		if has (line, '"event":"result"') then result_row = result_row or line end
		if failure_row == nil and has (line, '"event":"engine_failure"') and has (line, '"i":' .. CASE .. ',') then failure_row = line end
	end
	if result_row then
		write_record (false, "contradictory_result_row")
		refuse ("contradictory-result: the delegate wrote a result row (" .. result_row .. ") although the campaign is said to have stopped: " .. delegate_error)
	end
	if failure_row == nil then
		write_record (false, "setup_failure_no_engine_failure")
		refuse ("SETUP FAILURE: the delegate failed but wrote no case " .. CASE .. " engine_failure row, so the failure is a delegate, output or witness failure and not the engine's stop: " .. delegate_error)
	end
	if not has (failure_row, MARKER) then
		write_record (false, "wrong_boundary")
		refuse ("wrong-boundary: the engine stopped in case " .. CASE .. ", but not at " .. MARKER .. ": " .. failure_row)
	end
	if not string.find (delegate_error, "^env%-sound%-lifetime: env_case%(14, " .. CASE .. "%) FAILED in the engine") then
		write_record (false, "setup_failure_delegate_error_is_not_the_engine_stop")
		refuse ("SETUP FAILURE: the delegate's error is not its report of the engine's stop in case " .. CASE .. ": " .. delegate_error)
	end
	write_record (false, "stopped_at_boundary", string.format ('"marker":"%s","stopped":true,', MARKER))
	io.stderr:write (string.format ("%s: mode=%s stopped at %s after %d host request(s) for '%s'\n", NAME, mode, MARKER, requests, OP))
	return
end

-- ---------------------------------------------------------------------------------------------------------------- healthy and changed: the case must have completed, and the real case must show the host's answer
if not ran then
	write_record (false, "setup_failure_delegate_failed")
	error (tostring (problem), 0)
end
local completed, calls, kills, creation_word = case_result ()
if not completed then
	write_record (false, "setup_failure_case_did_not_complete")
	refuse ("SETUP FAILURE: the delegate returned but case " .. CASE .. " did not complete (refused, or no result row on " .. out_path .. "): the lifetime was not exercised, so there is no ownership verdict")
end
local facts = string.format ('"calls":%s,"creation_life_w":"%s","kills":%s,', tostring (calls), tostring (creation_word), tostring (kills))
if requests == 0 then
	write_record (true, "sound_host_capability_missing", facts)
	refuse ("sound-host-capability-missing: case " .. CASE .. " completed, and the host was asked 0 times for '" .. OP .. "' (" .. forwarded .. " other requests were forwarded unchanged): the campaign decided the lifetime without calling the host")
end
if mode == "healthy" and (calls ~= 3 or kills ~= 1) then
	write_record (true, "ownership_not_shown", facts)
	refuse ("ownership-not-shown: the reference host was asked " .. requests .. " time(s) but the real case shows " .. tostring (calls) .. " calls and " .. tostring (kills) .. " kills, not the three calls and one kill the original measured")
end
if mode == "changed" and (calls ~= 3 or kills ~= 0) then
	write_record (true, "ownership_not_shown", facts)
	refuse ("ownership-not-shown: the host never expired the effect (" .. requests .. " request(s)) but the real case shows " .. tostring (calls) .. " calls and " .. tostring (kills) .. " kills: the campaign killed an entity the host did not")
end
if mode == "changed_init" and creation_word ~= "3e800000" then
	write_record (true, "ownership_not_shown", facts)
	refuse ("ownership-not-shown: the host gave the new effect the lifetime 0.25 (" .. requests .. " request(s)) but the real constructor state holds the word " .. tostring (creation_word) .. ", not 3e800000: the campaign summed the samples itself")
end
write_record (true, mode == "healthy" and "host_asked_and_decided" or "host_changed_the_decision", facts)
io.stderr:write (string.format ("%s: mode=%s case %d completed; the host was asked %d time(s) for '%s'; the case shows %s calls, %s kills, creation word %s; %d other request(s) forwarded unchanged\n", NAME, mode, CASE, requests, OP, tostring (calls), tostring (kills), tostring (creation_word), forwarded))
