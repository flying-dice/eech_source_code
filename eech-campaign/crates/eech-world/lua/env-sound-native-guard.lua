-- TEST-ONLY GUARD DRIVER of the CANDIDATE native-handle boundary (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. A small SEPARATE file beside lua/env-sound-handle-guard.lua (the valid-expiry status stop, which stays
-- as it is) and the original drivers lua/env-sound-handle.lua and lua/env-sound-route.lua (the frozen 2e2fd548a evidence, which stay as they are).
--
-- WHAT CHANGED IN THE ENGINE (the candidate native, not this file). The Core no longer contains the legacy sound backend (modules/sound: soundman.c, dsound.c, dirsound.c, dsound3d.c, soundhlp.c). appsound/snd_nobackend.c gives the
-- no-backend answers the legacy code already handled (initialisation and the device list fail, a create gives NULL, the handle set is empty) and REFUSES BY NAME every request that would ask a native handle, the listener or the
-- sample store to do something: "[env-sound:unsupported_native_handle]" (a handle) or "[env-sound:unsupported_native_backend]" (samples, listener, handle table), before any native query, playback, destruction or state
-- change; and the lifetime wrapper (ss_hostenv.c) refuses the host's kill or unlink decision for an effect that holds a handle, before it writes, kills, unlinks or destroys, with the same handle tag. The hosted
-- SoundEffect playback entry (se_snd.c play_local_sound) refuses an effect that holds a handle with the same tag after the Core option and locality gates and BEFORE the range, amplification, channel, canopy and
-- interior mixing and the radio-noise entropy (that physical mixer is compiled only for the original game), so case 24 stops at that ENTRY, earlier than any handle operation of the adapter.
--
-- WHAT THIS DRIVER ASKS. It runs ONE of the fixture's existing cases 23 and 24 (csrc/eech_env_fixture.c env_snd_route_case: the test-owned typed static record with no device, the real pause and the real server routine for 23,
-- the real play dispatch for 24) once, through the real public host Lua path (Engine env_case), and accepts exactly ONE outcome: the engine's own fatal error whose FIRST LINE is, at its start, the one known shape of the
-- engine's fatal reaching Lua (EngineError::Fatal shown by eech-dc's lua_error and mlua: "runtime error: eech: EECH fatal error: host environment: ") directly followed by [env-sound:unsupported_native_handle] and a space.
-- The tag is judged at that STRUCTURAL position and nowhere else; a bare tag, a generic Lua error or another native fatal that quotes it, another tag, an unwrapped token, a repeated tag, a normal report (the frozen original
-- native, which serves the handle), a refusal of the case, a missing method or an output failure FAILS. After the stop the engine is poisoned: NO further engine call is made, nothing is censused, restored or released, the
-- typed static record stays valid through the fatal and the process ends, and neither a result of the control nor that the campaign continued is claimed. An expected stop is an `expected_refusal` row and the process exits 0.
--
-- PRIVACY. The engine's whole fatal text is kept in `<out>.fatal.raw` (PRIVATE, written BEFORE it is judged); the JSON holds only the named boundary and fixed words, never the text. Both evidence files are created
-- exclusively before anything is booted, every write and flush is checked, and BOTH files are closed here, a close that fails failing the run.
--
-- host.args: root=<installation root> out=<new JSON Lines path> case=<23|24> [scenario=lebanon_retail|georgia_retail]
-- Needs a module built with the env-observer and env-fixture features. One fresh process and root per run.

io.stdout:setvbuf ("no")

local args = host.args
local function refuse (message) error ("env-sound-native-guard: " .. message, 0) end

local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local LABELS = { [23] = "handle_invalid_server_unlink", [24] = "handle_direct_playback" }
local case = tonumber (args.case)
if not case or case ~= math.floor (case) or LABELS [case] == nil then refuse ("case must be 23 or 24, got '" .. tostring (args.case) .. "'") end
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
local MARKER = "[env-sound:unsupported_native_handle]"
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
	hard ("case " .. case .. " returned a report instead of stopping at " .. MARKER .. ": the engine served the native handle, or the control never reached one")
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
emit { event = "expected_refusal", case = case, label = LABELS [case], boundary = "unsupported_native_handle", engine_continued = false,
	note = "the engine stopped in its fatal handler at the first native handle request of the handle-holding effect (case 24: the entry of the hosted playback request, before its mixing); nothing was queried, censused, restored or released after it, and no result of the campaign continuing is claimed" }
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
io.stderr:write (string.format ("env-sound-native-guard: case=%d expected_named_refusal=unsupported_native_handle\n", case))
