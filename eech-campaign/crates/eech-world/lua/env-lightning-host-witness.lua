-- TEST-ONLY PUBLIC-HOST OWNERSHIP WITNESS of the lightning strike queue add (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. A script only: no C, no ABI, no provider and no host solver is written here.
--
-- THE QUESTION. The original `add_3d_lightning_strike` wrote the campaign's own private queue. The physical queue is now the HOST's: the add reaches the host through the public path (host-owned Lua ->
-- `eech_dc` -> the campaign -> the host's provider callback) as the physical operation `lightning_add`, and the host answers the slot it filled and the whole queue (docs/d3-lightning-queue.md and
-- eech-world/src/env.rs have the request and the reply). This witness asks, on the native it is run against, whether the campaign really asks the host and whether the host's ANSWER is what the real queue shows.
--
-- WHAT IT DOES. It wraps the `host.env_provider` factory BEFORE the delegate boots and counts ONLY the requests whose `op` is `lightning_add`; every other request is forwarded UNCHANGED to the original
-- provider. What it does with a `lightning_add` depends on the mode. It then runs the EXISTING primitive driver `lua/env-lightning-queue.lua` for the real case 2 (the original add with the main environment active;
-- the measured server has the main environment, so no prerequisite is forced) through `dofile`, so the delegate's argument checks, report validation, raw rows and output-failure handling are exactly its own,
-- with no second parser. The delegate's rows stay on `out` untouched; this witness writes ONE compact separate record of its own on `authority_out`.
--
-- MODES (`mode=`, default healthy):
--   healthy    the request is forwarded unchanged to the reference host, which queues the strike in its first free slot. REQUIRED: case 2 completed, the host was asked at least once and the real case shows ONE strike
--              queued (the case's own result row: occupancy 1, mask 00000001). Against the pre-extraction native the campaign queued the strike itself and the host was asked 0 times: the error
--              `queue-host-capability-missing` (the RED of the capability, not a physical regression).
--   changed    the host answers the request itself with a valid answer that differs from the reference: no strike is queued (slot -1, an empty queue: what a host does when it drops the add). REQUIRED: the host
--              was asked, and the REAL case shows NOTHING queued (occupancy 0, mask 00000000): the campaign queued only what the host said. A request count alone proves nothing: a native that asks the host
--              and still queues itself fails here.
--   withheld   the host raises an error. REQUIRED: the campaign STOPS at `[env-provider:lightning_add.callback]` (the delegate fails in the engine, no result row), after exactly the host action that was performed.
--   malformed  the host answers a slot that does not exist. REQUIRED: the campaign stops at `[env-provider:lightning_add.answer]`, likewise.
--   frames_without_fork   (case 4, the primitive control that ages a CLOUD BURST) the host answers the first `lightning_update` with a request for the strike animation's frame count although the queue holds no
--              forked strike. REQUIRED: the campaign refuses it at `[env-provider:lightning_update.answer]` BEFORE it touches the texture model (the actual C consumer), same stop evidence as withheld.
-- THE STOP EVIDENCE is the delegate's OWN record, read back when the delegate fails: the case's `engine_failure` row at the expected marker and NO result row, and the delegate's error must be its report of
-- the engine's stop. A delegate, output or witness failure whose message happens to carry the marker (a path, a diagnostic) is rejected as that failure, and a stale or contradictory result row is
-- `contradictory-result`; the record is `completed=false` and never a physical pass.
-- A stop is the expected evidence of withheld and malformed, not a completed success and not a fallback: the record says `stopped_at_boundary`, how many host requests were made before the stop, and that no
-- result row exists. A campaign that completes anyway, or stops at another marker, is an error.
--
-- VERDICTS (errors begin `env-lightning-host-witness:`): `SETUP FAILURE` -- the delegate failed (healthy, changed) or case 2 was refused or has no result: NOT a verdict on ownership (the add was not exercised);
-- `queue-host-capability-missing` -- case 2 completed and the host was asked 0 times; `ownership-not-shown` -- the host was asked but the real case does not show the host's answer; `not-stopped` /
-- `wrong-boundary` -- a withheld or malformed answer did not stop the campaign at its marker.
--
-- WHAT IT DOES NOT. It asserts no private type, struct layout or ABI, picks no physical outcome beyond the reference host's own first-free add, and exercises no renderer, speaker, network, sound entity or
-- lifetime. It says nothing of the other physical responsibilities of D3.
--
-- ARGUMENTS (host.args, as the host executable passes them; the witness is the SCRIPT the host runs and the delegate is loaded from a path):
--   eech-world <this script> root=<installation root> out=<delegate JSON Lines path> delegate=<ABSOLUTE path of lua/env-lightning-queue.lua> [mode=healthy|changed|withheld|malformed|frames_without_fork] [case=2 (4 for frames_without_fork)]
--                                                                                  [scenario=lebanon_retail|georgia_retail] [authority_out=<this witness's record path; default <out>.authority.jsonl>]
--   `root`, `out`, `scenario` are the delegate's own arguments and are passed through unchanged (the delegate reads the same `host.args`); `case` must be 2 (default 2). One fresh process and root per run.

io.stdout:setvbuf ("no")

local NAME = "env-lightning-host-witness"
local args = host.args
local function refuse (message) error (NAME .. ": " .. message, 0) end

local delegate_path = args.delegate
if type (delegate_path) ~= "string" or delegate_path == "" then refuse ("delegate=<absolute path of env-lightning-queue.lua> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<the delegate's JSON Lines path> is required") end
local mode = args.mode == nil and "healthy" or args.mode
-- the operation the mode is about, the primitive case that exercises it, and, for a mode that must stop the campaign, the boundary it must stop at
local MODES = {
	healthy = { op = "lightning_add", case = 2 },
	changed = { op = "lightning_add", case = 2 },
	withheld = { op = "lightning_add", case = 2, marker = "[env-provider:lightning_add.callback]" },
	malformed = { op = "lightning_add", case = 2, marker = "[env-provider:lightning_add.answer]" },
	frames_without_fork = { op = "lightning_update", case = 4, marker = "[env-provider:lightning_update.answer]" },
}
local plan = MODES [mode]
if plan == nil then refuse ("mode must be healthy, changed, withheld, malformed or frames_without_fork, got '" .. tostring (mode) .. "'") end
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

-- a valid queue answer holding nothing: 32 slots, none in use
local function empty_queue ()
	local queue = {}
	for i = 1, 32 do queue [i] = { in_use = false, type = 0, x = 0, y = 0, z = 0, time_started = 0, time_length = 0, frame = 0 } end
	return queue
end

local wrapped = function (...)
	local base = factory (...)
	if type (base) ~= "function" then error (NAME .. ": host.env_provider () did not return a provider function", 0) end
	return function (request, ...)
		if type (request) == "table" and request.op == OP then
			requests = requests + 1
			if mode == "changed" then return { slot = -1, queue = empty_queue () } end
			if mode == "withheld" then error ("the host withholds its lightning_add answer", 0) end
			if mode == "malformed" then return { slot = 40, queue = {} } end
			-- an update answered with a request for the strike animation's frame count, whatever the queue holds (the queue of the primitive control holds a cloud burst only)
			if mode == "frames_without_fork" then return { need_frames = true } end
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

-- the delegate's result row of this mode's case, if it wrote one: (completed, occupancy_after, mask_after_w)
local function case_result ()
	local rows = delegate_rows ()
	if rows == nil then return false end
	for _, line in ipairs (rows) do
		if has (line, '"event":"result"') and has (line, '"case":' .. CASE .. ',') and has (line, '"refused":false') then
			return true, string.match (line, '"occupancy_after":"(%d+)"'), string.match (line, '"mask_after_w":"(%x+)"')
		end
	end
	return false
end

-- ---------------------------------------------------------------------------------------------------------------- withheld, malformed and frames_without_fork: the campaign must STOP at the named boundary
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
	-- the stop is proved by the delegate's OWN record, not by a substring of the error: its case's engine_failure row at the expected boundary, and no result row. A delegate, output or witness
	-- failure (whose message may carry the marker, in a path for example) is rejected as what it is.
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
	if not string.find (delegate_error, "^env%-lightning%-queue: env_case%(13, " .. CASE .. "%) FAILED in the engine") then
		write_record (false, "setup_failure_delegate_error_is_not_the_engine_stop")
		refuse ("SETUP FAILURE: the delegate's error is not its report of the engine's stop in case " .. CASE .. ": " .. delegate_error)
	end
	write_record (false, "stopped_at_boundary", string.format ('"marker":"%s","stopped":true,', MARKER))
	io.stderr:write (string.format ("%s: mode=%s stopped at %s after %d host request(s) for '%s'\n", NAME, mode, MARKER, requests, OP))
	return
end

-- ---------------------------------------------------------------------------------------------------------------- healthy and changed: case 2 must have completed, and the real case must show the host's answer
if not ran then
	write_record (false, "setup_failure_delegate_failed")
	error (tostring (problem), 0)
end
local completed, occupancy, mask = case_result ()
if not completed then
	write_record (false, "setup_failure_case_2_did_not_complete")
	refuse ("SETUP FAILURE: the delegate returned but case 2 did not complete (refused, or no result row on " .. out_path .. "): the add was not exercised, so there is no ownership verdict")
end
local facts = string.format ('"mask_after_w":"%s","occupancy_after":"%s",', tostring (mask), tostring (occupancy))
if requests == 0 then
	write_record (true, "queue_host_capability_missing", facts)
	refuse ("queue-host-capability-missing: case 2 completed on the original add, and the host was asked 0 times for '" .. OP .. "' (" .. forwarded .. " other requests were forwarded unchanged): the campaign queued without calling the host")
end
if mode == "healthy" and (occupancy ~= "1" or mask ~= "00000001") then
	write_record (true, "ownership_not_shown", facts)
	refuse ("ownership-not-shown: the host was asked " .. requests .. " time(s) and queued the strike, but the real case shows occupancy " .. tostring (occupancy) .. " and mask " .. tostring (mask) .. ", not one strike in the first slot")
end
if mode == "changed" and (occupancy ~= "0" or mask ~= "00000000") then
	write_record (true, "ownership_not_shown", facts)
	refuse ("ownership-not-shown: the host dropped the add (" .. requests .. " request(s)) but the real case shows occupancy " .. tostring (occupancy) .. " and mask " .. tostring (mask) .. ": the campaign queued a strike the host did not")
end
write_record (true, mode == "healthy" and "host_asked_and_queued" or "host_changed_the_queue", facts)
io.stderr:write (string.format ("%s: mode=%s case 2 completed; the host was asked %d time(s) for '%s'; the case shows occupancy %s mask %s; %d other request(s) forwarded unchanged\n", NAME, mode, requests, OP, tostring (occupancy), tostring (mask), forwarded))
