-- TEST-ONLY PUBLIC-HOST CAPABILITY WITNESS of the lightning strike queue add (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. A script only: no C, no ABI, no provider and no host solver is written here.
--
-- THE QUESTION. The original `add_3d_lightning_strike` writes the campaign's own private queue. If the physical queue is to be the HOST's, the add must reach the host through the public path (host-owned Lua ->
-- `eech_dc` -> the campaign -> the host's provider callback) as a physical operation. This witness asks whether it does, on the native it is run against. The proposed semantic operation is `lightning_add` on the
-- existing `host.env_provider` vocabulary; its request and reply details follow the implementation and are NOT fixed, assumed or asserted here.
--
-- WHAT IT DOES. It wraps the `host.env_provider` factory BEFORE the delegate boots, forwards every request to the original provider UNCHANGED (the healthy / control path: nothing is altered or invented, so a host
-- that does not know `lightning_add` answers as it always did), and counts ONLY the requests whose `op` is `lightning_add`. It then runs the EXISTING primitive driver `lua/env-lightning-queue.lua` for the real
-- case 2 (the original add with the main environment active; the measured server has the main environment, so no prerequisite is forced) through `dofile`, so the delegate's argument checks, report validation, raw
-- rows and output-failure handling are exactly its own, with no second parser. The delegate's rows stay on `out` untouched; this witness writes ONE compact separate record of its own on `authority_out`.
--
-- THE VERDICT. After the delegate returned, the witness reads the delegate's own result row and requires that case 2 genuinely COMPLETED (not refused). Only then does the count mean anything:
--   * the delegate failed, or case 2 was refused or has no result:  error `env-lightning-host-witness: SETUP FAILURE ...` -- NOT a capability verdict (the add was never exercised);
--   * case 2 completed and the host was asked 0 times:             error `env-lightning-host-witness: queue-host-capability-missing: ...` -- the campaign queued WITHOUT calling the host;
--   * case 2 completed and the host was asked at least once:       success; the record carries the count.
-- Against the pre-extraction native the original add queues the strike itself (the delegate reports it) and the host is never asked: that is the intended RED of the NEW capability, not a physical regression.
--
-- WHAT IT DOES NOT. It does not alter, withhold or malform any answer, does not pick a reply grammar and does not assert any private type, struct layout or ABI. Changed / withheld / malformed host answers
-- and the host queue's own facts come with the implementation, in this same file. It exercises no renderer, speaker, network, sound entity or lifetime, and says nothing of the other physical responsibilities of D3.
--
-- ARGUMENTS (host.args, as the host executable passes them; the witness is the SCRIPT the host runs and the delegate is loaded from a path):
--   eech-world <this script> root=<installation root> out=<delegate JSON Lines path> delegate=<ABSOLUTE path of lua/env-lightning-queue.lua> [case=2] [scenario=lebanon_retail|georgia_retail]
--                                                                                  [authority_out=<this witness's record path; default <out>.authority.jsonl>]
--   `root`, `out`, `scenario` are the delegate's own arguments and are passed through unchanged (the delegate reads the same `host.args`); `case` must be 2 (default 2): the witness is for the add.

io.stdout:setvbuf ("no")

local NAME = "env-lightning-host-witness"
local OP = "lightning_add"
local args = host.args
local function refuse (message) error (NAME .. ": " .. message, 0) end

local delegate_path = args.delegate
if type (delegate_path) ~= "string" or delegate_path == "" then refuse ("delegate=<absolute path of env-lightning-queue.lua> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<the delegate's JSON Lines path> is required") end
if args.case ~= nil and tostring (args.case) ~= "2" then refuse ("case must be 2 (the original add with the main environment), got '" .. tostring (args.case) .. "'") end
args.case = "2"
local authority_path = args.authority_out
if authority_path == nil then authority_path = out_path .. ".authority.jsonl" end
if type (authority_path) ~= "string" or authority_path == "" or authority_path == out_path then refuse ("authority_out must be a path other than out") end

-- ---------------------------------------------------------------------------------------------------------------- the wrapper: forward everything, count only the op
local requests, forwarded = 0, 0
local factory = host.env_provider
if type (factory) ~= "function" then refuse ("this host has no host.env_provider factory") end
local wrapped = function (...)
	local base = factory (...)
	if type (base) ~= "function" then error (NAME .. ": host.env_provider () did not return a provider function", 0) end
	return function (request, ...)
		if type (request) == "table" and request.op == OP then
			requests = requests + 1
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

local function write_record (completed, verdict)
	local file, err = io.open (authority_path, "w")
	if not file then refuse ("OUTPUT FAILURE: opening " .. authority_path .. " failed: " .. tostring (err)) end
	local line = string.format ('{"case":2,"completed":%s,"delegate_ran":%s,"forwarded_other_requests":%d,"op":"%s","requests":%d,"source":"host.env_provider wrapper on the public path, forwarded unchanged","verdict":"%s"}\n',
		completed and "true" or "false", ran and "true" or "false", forwarded, OP, requests, verdict)
	local ok, werr = file:write (line)
	if not ok then refuse ("OUTPUT FAILURE: writing " .. authority_path .. " failed: " .. tostring (werr)) end
	ok, werr = file:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. authority_path .. " failed: " .. tostring (werr)) end
	ok, werr = file:close ()
	if not ok then refuse ("OUTPUT FAILURE: closing " .. authority_path .. " failed: " .. tostring (werr)) end
end

-- did the delegate itself finish, and did case 2 genuinely complete (its own result row, not refused)?
local completed = false
if ran then
	local text = read_all (out_path)
	if text ~= nil then
		for line in string.gmatch (text, "[^\n]+") do
			if string.find (line, '"event":"result"', 1, true) and string.find (line, '"case":2', 1, true) and string.find (line, '"refused":false', 1, true) then completed = true end
		end
	end
end

if not ran then
	write_record (false, "setup_failure_delegate_failed")
	error (tostring (problem), 0)
end
if not completed then
	write_record (false, "setup_failure_case_2_did_not_complete")
	refuse ("SETUP FAILURE: the delegate returned but case 2 did not complete (refused, or no result row on " .. out_path .. "): the add was not exercised, so there is no capability verdict")
end
if requests == 0 then
	write_record (true, "queue_host_capability_missing")
	refuse ("queue-host-capability-missing: case 2 completed on the original add, and the host was asked 0 times for '" .. OP .. "' (" .. forwarded .. " other requests were forwarded unchanged): the campaign queued without calling the host")
end
write_record (true, "host_asked")
io.stderr:write (string.format ("%s: case 2 completed; the host was asked %d time(s) for '%s'; %d other request(s) forwarded unchanged\n", NAME, requests, OP, forwarded))
