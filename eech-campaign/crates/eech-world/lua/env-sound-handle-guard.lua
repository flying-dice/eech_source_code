-- TEST-ONLY GUARD DRIVER of the CANDIDATE lifetime route (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. It is a SEPARATE, small file beside the ORIGINAL handle-status driver (lua/env-sound-handle.lua), which
-- stays as it is and is the only driver the 4238a35e6 native was characterized with.
--
-- WHAT CHANGED IN THE ENGINE (the candidate native, not this file). se_updt.c no longer calls the legacy sound system for the status of a native playback handle: campaign_sound_lifetime_update (ss_hostenv.c) takes only
-- whether a handle is held, and a host that asks whether the handle is playing at the expiry of a server effect stops the engine by name, "[env-sound:unsupported_handle_status]", BEFORE any native status query, any
-- lifetime write, kill, unlink or destruction. That is a fatal error of the engine, which poisons it. The headless route creates no native handle, so only a fixture that installs one can reach it.
--
-- WHAT THIS DRIVER ASKS. It runs ONE of the fixture's existing cases 21 and 22 (csrc/eech_env_fixture.c env_snd_handle_case, unchanged: the test-owned buffer-absent handle record and the real server routine) once, through the
-- real public host Lua path (Engine env_case), and accepts exactly ONE outcome: the engine's own fatal error whose FIRST LINE is, at its start, the one known shape of the engine's fatal reaching Lua (EngineError::Fatal shown by
-- eech-dc's lua_error and mlua: "runtime error: eech: EECH fatal error: host environment: ") directly followed by [env-sound:unsupported_handle_status] and a space. The tag is judged at that STRUCTURAL position and nowhere
-- else: a bare tag, a generic Lua error that quotes it, another native fatal (a callback or provider error, a different named stop) whose detail quotes it, a tag on a later line, an unwrapped token and a repeated tag all FAIL.
-- A normal report (the original native, which still answers the status and goes on to kill), a refusal of the case, any other fatal error, a missing method or an output failure FAILS the driver too. After the stop the engine is poisoned: this driver makes NO further engine call, takes
-- no census, restores nothing and claims neither that the campaign continued nor a result of the control; the static test-owned record stays valid through the fatal and the process ends, and the fixture's cleanup does
-- not run. An expected stop is reported as an `expected_refusal` row and the process exits 0; that is the outer diagnostic only.
--
-- PRIVACY. The engine's whole fatal text is kept in `<out>.fatal.raw` (PRIVATE, written BEFORE it is judged); the JSON file holds only the named boundary and a few fixed words, never the text, so no index, pointer or
-- coordinate is published. Both evidence files are created exclusively before anything is booted, every write and flush is checked, and BOTH files are closed here whatever either close does; a close that fails fails the run.
--
-- host.args: root=<installation root> out=<new JSON Lines path> case=<21|22> [scenario=lebanon_retail|georgia_retail]
-- Needs a module built with the env-observer and env-fixture features. One fresh process and root per run.

io.stdout:setvbuf ("no")

local args = host.args
local function refuse (message) error ("env-sound-handle-guard: " .. message, 0) end

local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local LABELS = { [21] = "handle_idle_buffer_absent_server_expiry", [22] = "handle_idle_buffer_absent_stale_playing_server_expiry" }
local case = tonumber (args.case)
if not case or case ~= math.floor (case) or LABELS [case] == nil then refuse ("case must be 21 or 22, got '" .. tostring (args.case) .. "'") end
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario_name = args.scenario == nil and "lebanon_retail" or args.scenario
local scenario = scenarios [scenario_name]
if scenario == nil then refuse ("scenario must be lebanon_retail or georgia_retail, got '" .. tostring (scenario_name) .. "'") end

-- every evidence file is created exclusively, here, before anything is booted
local create = host.output_create_new
if type (create) ~= "function" then refuse ("this host has no host.output_create_new: evidence files cannot be created exclusively, and the driver does not fall back to a truncating open") end
local paths = { out = out_path, fatal = out_path .. ".fatal.raw" }
local files, opened = {}, {}
for _, name in ipairs ({ "out", "fatal" }) do
	local file, why = create (paths [name])
	if file == nil then
		for _, done in ipairs (opened) do pcall (files [done].close, files [done]) end
		refuse ("OUTPUT FAILURE: creating " .. paths [name] .. " failed: " .. tostring (why) .. " (evidence is created exclusively and never overwritten)")
	end
	files [name] = file
	opened [#opened + 1] = name
end

local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
local function json_string (s)
	return '"' .. (string.gsub (s, '[%c"\\\128-\255]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)) .. '"'
end
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
	local ok, err = files.out:write ("{" .. table.concat (parts, ",") .. "}\n")
	if not ok then refuse ("OUTPUT FAILURE: writing " .. out_path .. " failed: " .. tostring (err)) end
	ok, err = files.out:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. out_path .. " failed: " .. tostring (err)) end
end
local function hard (message)
	pcall (emit, { event = "witness_failure", reason = message })
	refuse (message)
end
-- the private raw text, written BEFORE it is judged
local function keep (text)
	local ok, err = files.fatal:write (text)
	if not ok then refuse ("OUTPUT FAILURE: writing " .. paths.fatal .. " failed: " .. tostring (err)) end
	ok, err = files.fatal:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. paths.fatal .. " failed: " .. tostring (err)) end
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
-- armed BEFORE boot exactly as the original driver does, so the only difference between the two runs is the engine's code
if type (dc.env_observer_arm) ~= "function" then
	emit { event = "refusal", reason = "env_observer_unavailable" }
	refuse ("the module has no env_observer_arm function: build eech-dc with the env-observer feature")
end
dc.env_observer_arm (true)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign,
	gunship = "apache", seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, case = case, label = LABELS [case], group = 14 }
local present, method = pcall (function () return engine.env_case end)
if not present or type (method) ~= "function" then
	emit { event = "refusal", reason = "env_case_unavailable" }
	refuse ("the engine has no env_case method: build eech-dc with the env-observer and env-fixture features")
end

-- ---------------------------------------------------------------------------------------------------------------- the one accepted outcome: the named stop
local MARKER = "[env-sound:unsupported_handle_status]"
-- the first line of the engine's fatal as it reaches Lua (the same wrapper the retained parent-play overload stop shows): mlua's "runtime error: " around eech-dc's "eech: " around EngineError::Fatal's "EECH fatal error: "
-- around the engine's own message, which begins with "host environment: " (ss_hostenv.c debug_fatal)
local WRAPPER = "runtime error: eech: EECH fatal error: host environment: "
local ok, text = pcall (engine.env_case, engine, 14, case)
keep (tostring (text) .. "\n")
if ok then
	if type (text) == "string" and string.find (text, "refused=", 1, true) then
		emit { event = "control_refused", case = case }
		hard ("the fixture REFUSED case " .. case .. ": the control did not run, an unavailable prerequisite and not the named stop")
	end
	hard ("case " .. case .. " returned a report instead of stopping at " .. MARKER .. ": the engine answered or never asked the status of the native handle")
end
local message = tostring (text)
local first_line = string.match (message, "^[^\n]*")
if string.sub (first_line, 1, #WRAPPER + #MARKER + 1) ~= WRAPPER .. MARKER .. " " then
	-- only the bracketed tag at the same structural position is projected; no text of the failure reaches the JSON (it stays in the private raw)
	local position_tag = string.match (first_line, "^runtime error: eech: EECH fatal error: host environment: (%[[%w_%-]+:[%w_%.]+%])")
	emit { event = "engine_failure", case = case, tag = position_tag or "none" }
	hard ("the case stopped, but not with the known engine fatal wrapper naming " .. MARKER .. " first (the whole text is in the private raw)")
end
if string.find (message, MARKER, #WRAPPER + #MARKER + 1, true) ~= nil then hard ("the stop names " .. MARKER .. " more than once (the whole text is in the private raw)") end
-- NOTHING more is asked of the engine: it is poisoned, so no census, report, restoration or receiver check follows
emit { event = "expected_refusal", case = case, label = LABELS [case], boundary = "unsupported_handle_status", engine_continued = false,
	note = "the engine stopped in its fatal handler at the first expiry of the handle-holding effect; nothing was queried, censused, restored or released after it, and no result of the campaign continuing is claimed" }
emit { event = "result", case = case, outcome = "expected_named_refusal" }

-- BOTH evidence files are closed here, the private raw first so that a failure to close it can still be written to the JSON; every close is attempted, every failure is reported and the run then fails closed
local close_errors = {}
for position = #opened, 1, -1 do
	local name = opened [position]
	local closed, close_error = files [name]:close ()
	if not closed then
		close_errors [#close_errors + 1] = "closing " .. paths [name] .. " failed: " .. tostring (close_error)
		if name ~= "out" then pcall (emit, { event = "output_failure", reason = close_errors [#close_errors] }) end
	end
end
if #close_errors > 0 then refuse ("OUTPUT FAILURE: " .. table.concat (close_errors, "; ")) end
io.stderr:write (string.format ("env-sound-handle-guard: case=%d expected_named_refusal=unsupported_handle_status\n", case))
