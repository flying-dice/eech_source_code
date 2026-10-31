-- TEST-ONLY host-authority and refusal witness of the host-owned terrain-sector surface enumeration (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
-- Needs an eech-dc built with the `env-observer` feature (which includes `env-fixture`): the observer is what shows the scratch the REAL sound consumer read.
--
-- The campaign's session sound routine (ss_updt.c update_session_sound_effects) calls get_terrain_3d_types_in_sector (x, z) once per update, with the camera clamped to the map, and reads the
-- scratch terrain_types_in_sector it fills. In the headless campaign module that function is a request to the HOST's terrain provider (op "sector": the original's return value and the complete
-- flags by value, mb_hostterrain.c campaign_terrain_types_in_sector); the campaign keeps only a by-value mirror of the last reply, written after the answer has been checked in full. This driver
-- shows, through the public path (host-owned Lua -> the native module -> the host's provider -> back into the real sound routine), that the answer is the host's and that the routine used it:
--
--   control                  the host's own answers (host.terrain_provider), delegated unchanged and counted: the request of every sound update (position, season) and the first answers are recorded.
--   changed_sector           EVERY sector request is answered { returned = 11, flags = <six sound-relevant types set> }: a VALID, finite enumeration the host's map would not give. The scratch the
--                            sound routine read (the observer's hash and count of terrain_types_in_sector right after its call) must be exactly the flags that were sent.
--   withheld_sector          the provider raises:                                   [terrain-provider:sector.callback]
--   malformed_sector_flags   the flags are 61 characters, not 62:                    [terrain-provider:sector.flags]
--   malformed_sector_returned the return value is 0 with valid flags:               [terrain-provider:sector.returned]
--
-- The alteration applies to the sector operation ONLY (the map facts, the point and the fast queries are delegated unchanged and counted) from the first request of the run, and the real host is not
-- called for an altered sector question. The provider is the campaign's whole terrain world after boot: this is a bounded alternate-host-world input for the sector query, not an isolation of every
-- consumer. What it can show, and what it cannot, is in docs/d3-environment-sector-surface.md: it shows that a sector request is made once per sound update and that the scratch the routine read is what the host
-- answered (hash and count of the whole scratch); it does NOT observe the sound amplitudes the routine then sets on the session's sound children, and the render code that also reads the scratch (3drain.c) is
-- dormant here and is not inferred from this run. Nothing is claimed for a campaign with a player or a renderer.
--
-- Rows (JSON Lines): boot, request (the first requests: position, season, the answer sent or received), observer pair (the first scratch records of the routine), authority (the counters). A failure of the
-- ENGINE is an `authority` row with completed = false and the engine's first error line; a failure of the WITNESS (a clock that does not advance, a frame budget, a malformed observer line, an output
-- failure) raises an error beginning `env-sector-authority:` after a best-effort `witness_failure` row.
--
-- host.args: root, mode (default control), scenario (lebanon_retail default | georgia_retail), frames (5 to 600, default 30), frame_ms (10 to 1000, default 100), out (JSON Lines path)

io.stdout:setvbuf ("no")

local huge = math.huge
local function finite (v) return type (v) == "number" and v == v and v ~= huge and v ~= -huge end

local args = host.args
local function refuse (message) error ("env-sector-authority: " .. message, 0) end
local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local mode = args.mode or "control"
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario_name = args.scenario == nil and "lebanon_retail" or args.scenario
local scenario = scenarios [scenario_name]
if scenario == nil then refuse ("scenario must be lebanon_retail or georgia_retail, got '" .. tostring (scenario_name) .. "'") end
local frames_wanted, frame_ms = tonumber (args.frames or "30"), tonumber (args.frame_ms or "100")
if not (finite (frames_wanted) and finite (frame_ms)) or frames_wanted ~= math.floor (frames_wanted) or frames_wanted < 5 or frames_wanted > 600 or frame_ms < 10 or frame_ms > 1000 then
	refuse ("frames must be a whole number from 5 to 600 and frame_ms a number from 10 to 1000 (and both finite)")
end
local seed = 1
local out = assert (io.open (out_path, "w"))

local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
local function json_string (s)
	return '"' .. (string.gsub (s, '[%c"\\\128-\255]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)) .. '"'
end
-- every write and flush of the evidence is checked: a failed one is a failure of the witness, even after the last row
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

-- ---------------------------------------------------------------------------------------------------------------- the modes
local TYPES = 62
local function flags_with (set)
	local chars = {}
	for id = 0, TYPES - 1 do chars [#chars + 1] = set [id] and "1" or "0" end
	return table.concat (chars)
end
-- river 1, reservoir 2, sea 7, altered land 1 and 2 (21, 22), forest top 46: the types the sound routine reads (ss_updt.c)
local SENT_FLAGS = flags_with { [1] = true, [2] = true, [7] = true, [21] = true, [22] = true, [46] = true }
local SENT_RETURNED = 11
local MODES = {
	control = {},
	changed_sector = { answer = function () return { returned = SENT_RETURNED, flags = SENT_FLAGS } end },
	withheld_sector = { answer = function () error ("the host withholds its sector answer", 0) end, marker = "sector.callback" },
	malformed_sector_flags = { answer = function () return { returned = 3, flags = string.sub (SENT_FLAGS, 1, TYPES - 1) } end, marker = "sector.flags" },
	malformed_sector_returned = { answer = function () return { returned = 0, flags = SENT_FLAGS } end, marker = "sector.returned" },
}
local plan = MODES [mode]
if plan == nil then refuse ("unknown mode '" .. tostring (mode) .. "'") end

-- ---------------------------------------------------------------------------------------------------------------- the host's provider, counted, the sector question altered by the mode
local factory = assert (host.terrain_provider, "env-sector-authority: this host has no host.terrain_provider")
local map = scenario.map
local real = factory (root, map)
local counts = { map = 0, point = 0, fast = 0, sector = 0 }
local altered = 0
local REQUEST_LIMIT = 4
local requests = {}
local function provider (request)
	local op = request.op
	if counts [op] ~= nil then counts [op] = counts [op] + 1 end
	if op == "sector" and plan.answer then
		altered = altered + 1
		-- the answer is built first and recorded exactly as it is sent (a malformed one as malformed; a withheld one as "withheld"), then sent or raised
		local ok, answer = pcall (plan.answer)
		if #requests < REQUEST_LIMIT then
			requests [#requests + 1] = {
				n = counts.sector, x = string.format ("%.9g", request.x), z = string.format ("%.9g", request.z), winter = request.winter,
				returned = ok and tostring (answer.returned) or "withheld", flags = ok and tostring (answer.flags) or "withheld", sent = true,
			}
		end
		if not ok then error (answer, 0) end
		return answer
	end
	local answer = real (request)
	if op == "sector" and #requests < REQUEST_LIMIT and type (answer) == "table" then
		requests [#requests + 1] = { n = counts.sector, x = string.format ("%.9g", request.x), z = string.format ("%.9g", request.z), winter = request.winter, returned = tostring (answer.returned), flags = tostring (answer.flags), sent = false }
	end
	return answer
end

-- ---------------------------------------------------------------------------------------------------------------- boot
local dc = require ("eech_dc")
dc.prepare_installation (root)
-- the observer is armed BEFORE boot, so that no sound update is missed
if type (dc.env_observer_arm) ~= "function" then
	emit { event = "refusal", reason = "env_observer_unavailable", detail = "the module was built without the env-observer feature" }
	refuse ("the module has no env_observer_arm function: build eech-dc with the env-observer feature")
end
dc.env_observer_arm (true)
local engine = dc.boot {
	install_root = root, map = map, terrain = provider, environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache", seed = seed,
	bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss, supply_reached = host.supply_reached, line_of_sight = host.line_of_sight,
	sense = host.sense, aim = host.aim,
}
emit { event = "boot", mode = mode, scenario = scenario_name, seed = seed, frames = frames_wanted, frame_ms = frame_ms, feature = "env-observer" }

local clock = engine:clock ()
local function first_line (text) return (string.match (tostring (text), "[^\r\n]+") or tostring (text)) end
local frames = 0
local failure
local frame_budget = frames_wanted + 5
while frames < frames_wanted do
	if frames >= frame_budget then hard ("FRAME BUDGET: " .. frames .. " frames were run") end
	local was = clock.elapsed_seconds
	local ok, err = pcall (engine.frame, engine, frame_ms)
	if not ok then
		-- the engine's own failure: the witness's failures carry their own prefix and are never an expected refusal
		if string.find (tostring (err), "env-sector-authority:", 1, true) then error (tostring (err), 0) end
		failure = first_line (err)
		break
	end
	frames = frames + 1
	clock = engine:clock ()
	local now = clock.elapsed_seconds
	if not finite (now) or not (now > was) then hard ("STALLED CLOCK: a frame left the simulated clock at " .. tostring (now) .. " s from " .. tostring (was) .. " s") end
end

-- ---------------------------------------------------------------------------------------------------------------- the observer: what the sound routine's scratch held
-- Only a run the engine did not stop can be asked. Every line of the stream is `env-obs seq=N kind=K a=A b=B rng=R rng_w=H w0=H ... w5=H`; the sound facts (kind 30) come as pairs: the first has the
-- hash of the whole scratch (b, a signed int) and the camera words, the second (a = -1) the number of scratch entries set (b) and the clamped sector x/z words (w0, w1).
local SOUND_FACTS = 30
local function float_of_word (word)
	local n = tonumber (word, 16)
	local exponent, mantissa = math.floor (n / 8388608) % 256, n % 8388608
	local negative = n >= 2147483648
	local value
	if exponent == 255 then hard ("an observer word is not a finite float: " .. word)
	elseif exponent == 0 then value = mantissa * 2 ^ -149
	else value = (1 + mantissa / 8388608) * 2 ^ (exponent - 127) end
	return negative and -value or value
end
local pairs_seen, sound_calls, sound_records = {}, nil, nil
if not failure then
	local ok, report = pcall (engine.env_observer_report, engine)
	if not ok then hard ("OBSERVER REPORT UNREADABLE: " .. first_line (report)) end
	sound_calls, sound_records = tonumber (string.match (report, " sound_calls=(%d+)")), tonumber (string.match (report, " sound_records=(%d+)"))
	if not sound_calls or not sound_records then hard ("OBSERVER REPORT GRAMMAR: no sound_calls or sound_records in: " .. first_line (report)) end
	local ok2, stream = pcall (engine.env_observer_events, engine)
	if not ok2 then hard ("OBSERVER EVENTS UNREADABLE: " .. first_line (stream)) end
	local events = {}
	for line in string.gmatch (stream, "[^\n]+") do
		local seq, kind, a, b, w0, w1, w2 = string.match (line, "^env%-obs seq=(%d+) kind=(%d+) a=(%-?%d+) b=(%-?%d+) rng=%-?%d+ rng_w=%x+ w0=(%x+) w1=(%x+) w2=(%x+) w3=%x+ w4=%x+ w5=%x+$")
		if not seq then hard ("OBSERVER GRAMMAR: not an observer event line: " .. line) end
		if tonumber (kind) == SOUND_FACTS then events [#events + 1] = { a = tonumber (a), b = tonumber (b), w0 = w0, w1 = w1, w2 = w2, seq = tonumber (seq) } end
	end
	if #events % 2 ~= 0 then hard ("OBSERVER PAIRS: " .. #events .. " sound-fact events, not an even number") end
	for i = 1, #events, 2 do
		local scratch, tail = events [i], events [i + 1]
		if tail.a ~= -1 then hard ("OBSERVER PAIRS: the event after sound fact " .. scratch.seq .. " is not its second half") end
		pairs_seen [#pairs_seen + 1] = {
			hash = scratch.b < 0 and scratch.b + 4294967296 or scratch.b, nonzero = tail.b, sector_x = string.format ("%.9g", float_of_word (tail.w0)), sector_z = string.format ("%.9g", float_of_word (tail.w1)),
			in_cockpit = scratch.a,
		}
	end
end

for _, r in ipairs (requests) do
	emit { event = "request", n = r.n, x = r.x, z = r.z, winter = r.winter, returned = r.returned, flags = r.flags, sent = r.sent }
end
for i, p in ipairs (pairs_seen) do
	if i > 4 then break end
	emit { event = "pair", n = i, hash = p.hash, nonzero = p.nonzero, sector_x = p.sector_x, sector_z = p.sector_z, in_cockpit = p.in_cockpit }
end
emit {
	event = "authority", mode = mode, expected_marker = plan.marker or "", completed = failure == nil, failure = failure or "", frames = frames, frames_wanted = frames_wanted, elapsed = clock.elapsed_seconds,
	sector_requests = counts.sector, altered_sector = altered, map_requests = counts.map, point_requests = counts.point, fast_requests = counts.fast, request_rows = #requests,
	observer_checked = failure == nil, sound_calls = sound_calls or -1, sound_records = sound_records or -1, pairs = #pairs_seen, pair_rows = math.min (#pairs_seen, 4),
}
-- the last thing the witness does is close its evidence, and a close that fails is a failure after the success row as much as before it
local closed, close_error = out:close ()
if not closed then refuse ("OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error)) end
io.stderr:write (string.format ("env-sector-authority: mode=%s completed=%s frames=%d sector_requests=%d altered=%d sound_calls=%s failure=%s\n", mode, tostring (failure == nil), frames, counts.sector, altered,
	tostring (sound_calls), failure or "none"))
